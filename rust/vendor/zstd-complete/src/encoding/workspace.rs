//! Reusable slice-to-slice compression workspaces.

use alloc::vec::Vec;
use core::{fmt, marker::PhantomData, mem::MaybeUninit};

use super::{levels::c_port, CompressionLevel};
use crate::workspace::{Arena, ArenaError, ArenaSize, ReusableVec};

use c_port::{
    WorkspaceCctxParameters as CctxParameters, WorkspaceDFastMatchState as DFastMatchState,
    WorkspaceFastMatchState as FastMatchState, WorkspaceFrameBlockState as FrameBlockState,
    WorkspaceGreedyMatchState as GreedyMatchState, WorkspaceLazyBlockStrategy as LazyBlockStrategy,
    WorkspaceLdmWorkspace as LdmWorkspace, WorkspaceOptBlockState as OptBlockState,
    WorkspaceOptFrameStrategy as OptFrameStrategy, WorkspaceParamSwitch as ParamSwitch,
    WorkspaceStrategy as Strategy,
};

/// Errors reported while creating or using an encoder workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncoderWorkspaceError {
    InsufficientWorkspace { required: usize, provided: usize },
    InputTooLarge { maximum: usize, provided: usize },
    OutputTooSmall { required: usize, provided: usize },
    SizeOverflow,
}

impl fmt::Display for EncoderWorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientWorkspace { required, provided } => write!(
                formatter,
                "encoder workspace needs {required} bytes but only {provided} were provided"
            ),
            Self::InputTooLarge { maximum, provided } => write!(
                formatter,
                "encoder workspace accepts at most {maximum} input bytes but received {provided}"
            ),
            Self::OutputTooSmall { required, provided } => write!(
                formatter,
                "encoded output needs {required} bytes but only {provided} were provided"
            ),
            Self::SizeOverflow => formatter.write_str("encoder workspace size overflowed"),
        }
    }
}

// Boxing a strategy would add a heap allocation to caller-backed contexts;
// the enum deliberately keeps the selected state inline in the arena owner.
#[allow(clippy::large_enum_variant)]
enum MatchWorkspace {
    Uncompressed,
    Fast(FastMatchState),
    DFast(DFastMatchState),
    Greedy {
        state: GreedyMatchState,
        depth: LazyBlockStrategy,
    },
    Opt {
        state: OptBlockState,
        strategy: OptFrameStrategy,
    },
}

struct EncoderCore {
    level: i32,
    maximum_input: usize,
    base_cctx: CctxParameters,
    frame: FrameBlockState,
    matcher: MatchWorkspace,
    ldm: Option<LdmWorkspace>,
}

impl EncoderCore {
    fn add_size(
        size: &mut ArenaSize,
        level: i32,
        maximum_input: usize,
    ) -> Result<CctxParameters, EncoderWorkspaceError> {
        let cctx = CctxParameters::for_level(level, maximum_input as u64, 0);
        FrameBlockState::add_workspace_size(size).map_err(map_arena)?;
        let block_size = crate::common::MAX_BLOCK_SIZE as usize;
        if level == 0 {
            return Ok(cctx);
        }
        match cctx.compression.strategy {
            Strategy::Fast => {
                FastMatchState::add_workspace_size(size, cctx.compression, block_size)
                    .map_err(map_arena)?;
            }
            Strategy::DFast => {
                DFastMatchState::add_workspace_size(size, cctx.compression, block_size)
                    .map_err(map_arena)?;
            }
            Strategy::Greedy | Strategy::Lazy | Strategy::Lazy2 | Strategy::BtLazy2 => {
                GreedyMatchState::add_workspace_size(size, cctx.compression, block_size)
                    .map_err(map_arena)?;
            }
            Strategy::BtOpt | Strategy::BtUltra | Strategy::BtUltra2 => {
                OptBlockState::add_workspace_size(size, cctx.compression, block_size)
                    .map_err(map_arena)?;
            }
        }
        if cctx.ldm.enable_ldm == ParamSwitch::Enable {
            LdmWorkspace::add_workspace_size(size, cctx.ldm, maximum_input).map_err(map_arena)?;
        }
        Ok(cctx)
    }

    fn new_in(
        arena: &mut Arena<'_>,
        level: i32,
        maximum_input: usize,
        cctx: CctxParameters,
    ) -> Result<Self, EncoderWorkspaceError> {
        let block_size = crate::common::MAX_BLOCK_SIZE as usize;
        let frame = FrameBlockState::new_in(arena, cctx.compression, cctx.max_block_size)
            .map_err(map_arena)?;
        let matcher = if level == 0 {
            MatchWorkspace::Uncompressed
        } else {
            match cctx.compression.strategy {
                Strategy::Fast => MatchWorkspace::Fast(
                    FastMatchState::new_in(arena, cctx.compression, block_size)
                        .map_err(map_arena)?,
                ),
                Strategy::DFast => MatchWorkspace::DFast(
                    DFastMatchState::new_in(arena, cctx.compression, block_size)
                        .map_err(map_arena)?,
                ),
                strategy @ (Strategy::Greedy
                | Strategy::Lazy
                | Strategy::Lazy2
                | Strategy::BtLazy2) => MatchWorkspace::Greedy {
                    state: GreedyMatchState::new_in(arena, cctx.compression, block_size)
                        .map_err(map_arena)?,
                    depth: match strategy {
                        Strategy::Greedy => LazyBlockStrategy::Greedy,
                        Strategy::Lazy => LazyBlockStrategy::Lazy,
                        Strategy::Lazy2 => LazyBlockStrategy::Lazy2,
                        Strategy::BtLazy2 => LazyBlockStrategy::BtLazy2,
                        _ => unreachable!(),
                    },
                },
                strategy @ (Strategy::BtOpt | Strategy::BtUltra | Strategy::BtUltra2) => {
                    MatchWorkspace::Opt {
                        state: OptBlockState::new_in(arena, cctx.compression, block_size)
                            .map_err(map_arena)?,
                        strategy: match strategy {
                            Strategy::BtOpt => OptFrameStrategy::BtOpt,
                            Strategy::BtUltra => OptFrameStrategy::BtUltra,
                            Strategy::BtUltra2 => OptFrameStrategy::BtUltra2,
                            _ => unreachable!(),
                        },
                    }
                }
            }
        };
        let ldm = if cctx.ldm.enable_ldm == ParamSwitch::Enable {
            Some(LdmWorkspace::new_in(arena, cctx.ldm, maximum_input).map_err(map_arena)?)
        } else {
            None
        };
        Ok(Self {
            level,
            maximum_input,
            base_cctx: cctx,
            frame,
            matcher,
            ldm,
        })
    }

    fn encode_into<'output>(
        &mut self,
        input: &[u8],
        output: &'output mut [u8],
    ) -> Result<&'output [u8], EncoderWorkspaceError> {
        if input.len() > self.maximum_input {
            return Err(EncoderWorkspaceError::InputTooLarge {
                maximum: self.maximum_input,
                provided: input.len(),
            });
        }
        let required = checked_output_size(input.len())?;
        if output.len() < required {
            return Err(EncoderWorkspaceError::OutputTooSmall {
                required,
                provided: output.len(),
            });
        }
        let cctx = CctxParameters::from_compression_parameters(
            self.level,
            self.base_cctx.compression,
            input.len() as u64,
        );
        // SAFETY: the temporary vector is recovered before this function
        // returns and the capacity check above prevents growth.
        let reusable = unsafe { ReusableVec::from_static_parts(output.as_mut_ptr(), output.len()) };
        let (mut output_vec, lease) = reusable.lease_vec();
        let ldm = &mut self.ldm;
        match &mut self.matcher {
            MatchWorkspace::Uncompressed => encode_uncompressed(input, &mut output_vec),
            MatchWorkspace::Fast(state) => {
                c_port::workspace_encode_fast(input, cctx, &mut self.frame, state, &mut output_vec)
            }
            MatchWorkspace::DFast(state) => {
                c_port::workspace_encode_dfast(input, cctx, &mut self.frame, state, &mut output_vec)
            }
            MatchWorkspace::Greedy { state, depth } => c_port::workspace_encode_greedy(
                input,
                cctx,
                *depth,
                &mut self.frame,
                state,
                &mut output_vec,
            ),
            MatchWorkspace::Opt { state, strategy } => c_port::workspace_encode_opt(
                input,
                cctx,
                *strategy,
                &mut self.frame,
                state,
                ldm.as_mut(),
                &mut output_vec,
            ),
        }
        let length = output_vec.len();
        let reusable = ReusableVec::recover_vec(output_vec, lease);
        drop(reusable);
        Ok(&output[..length])
    }
}

fn encode_uncompressed(input: &[u8], output: &mut Vec<u8>) {
    use crate::{
        blocks::block::BlockType,
        encoding::{block_header::BlockHeader, frame_header::FrameHeader},
    };
    output.clear();
    FrameHeader {
        frame_content_size: Some(input.len() as u64),
        single_segment: true,
        content_checksum: false,
        dictionary_id: None,
        window_size: None,
    }
    .serialize(output);
    if input.is_empty() {
        BlockHeader {
            last_block: true,
            block_type: BlockType::Raw,
            block_size: 0,
        }
        .serialize(output);
        return;
    }
    let block_size = crate::common::MAX_BLOCK_SIZE as usize;
    for (index, block) in input.chunks(block_size).enumerate() {
        BlockHeader {
            last_block: (index + 1) * block_size >= input.len(),
            block_type: BlockType::Raw,
            block_size: block.len() as u32,
        }
        .serialize(output);
        output.extend_from_slice(block);
    }
}

/// An owned compression workspace reusable without further allocation.
pub struct EncoderWorkspace {
    core: EncoderCore,
    storage: Vec<MaybeUninit<u8>>,
}

impl EncoderWorkspace {
    pub fn new(
        level: CompressionLevel,
        maximum_input: usize,
    ) -> Result<Self, EncoderWorkspaceError> {
        let required = Self::required_size(level, maximum_input)?;
        let mut storage = Vec::with_capacity(required);
        storage.resize_with(required, MaybeUninit::uninit);
        let mut arena = Arena::new(&mut storage);
        let mut size = ArenaSize::new();
        let cctx = EncoderCore::add_size(&mut size, level.get(), maximum_input)?;
        let core = EncoderCore::new_in(&mut arena, level.get(), maximum_input, cctx)?;
        Ok(Self { core, storage })
    }

    pub fn required_size(
        level: CompressionLevel,
        maximum_input: usize,
    ) -> Result<usize, EncoderWorkspaceError> {
        let mut size = ArenaSize::new();
        EncoderCore::add_size(&mut size, level.get(), maximum_input)?;
        Ok(size.finish())
    }

    pub fn required_output_size(maximum_input: usize) -> Result<usize, EncoderWorkspaceError> {
        checked_output_size(maximum_input)
    }

    pub fn encode_into<'output>(
        &mut self,
        input: &[u8],
        output: &'output mut [u8],
    ) -> Result<&'output [u8], EncoderWorkspaceError> {
        self.core.encode_into(input, output)
    }

    pub fn workspace_bytes(&self) -> usize {
        self.storage.len()
    }

    pub const fn maximum_input(&self) -> usize {
        self.core.maximum_input
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EncoderWorkspaceError {}

/// A compression context whose working state lives entirely in caller storage.
pub struct StaticEncoderWorkspace<'storage> {
    core: EncoderCore,
    workspace_bytes: usize,
    marker: PhantomData<&'storage mut [MaybeUninit<u8>]>,
}

impl<'storage> StaticEncoderWorkspace<'storage> {
    pub fn new(
        storage: &'storage mut [u8],
        level: CompressionLevel,
        maximum_input: usize,
    ) -> Result<Self, EncoderWorkspaceError> {
        Self::new_uninit(initialized_bytes_as_uninit(storage), level, maximum_input)
    }

    /// Constructs a workspace without initializing caller-provided bytes.
    pub fn new_uninit(
        storage: &'storage mut [MaybeUninit<u8>],
        level: CompressionLevel,
        maximum_input: usize,
    ) -> Result<Self, EncoderWorkspaceError> {
        let required = EncoderWorkspace::required_size(level, maximum_input)?;
        if storage.len() < required {
            return Err(EncoderWorkspaceError::InsufficientWorkspace {
                required,
                provided: storage.len(),
            });
        }
        let mut size = ArenaSize::new();
        let cctx = EncoderCore::add_size(&mut size, level.get(), maximum_input)?;
        let mut arena = Arena::new(storage);
        let core = EncoderCore::new_in(&mut arena, level.get(), maximum_input, cctx)?;
        Ok(Self {
            core,
            workspace_bytes: storage.len(),
            marker: PhantomData,
        })
    }

    pub fn encode_into<'output>(
        &mut self,
        input: &[u8],
        output: &'output mut [u8],
    ) -> Result<&'output [u8], EncoderWorkspaceError> {
        self.core.encode_into(input, output)
    }

    pub const fn maximum_input(&self) -> usize {
        self.core.maximum_input
    }

    pub const fn workspace_bytes(&self) -> usize {
        self.workspace_bytes
    }
}

fn initialized_bytes_as_uninit(storage: &mut [u8]) -> &mut [MaybeUninit<u8>] {
    // SAFETY: `MaybeUninit<u8>` has the same layout as `u8`, and treating
    // initialized bytes as possibly uninitialized only weakens the invariant.
    unsafe { core::slice::from_raw_parts_mut(storage.as_mut_ptr().cast(), storage.len()) }
}

fn map_arena(error: ArenaError) -> EncoderWorkspaceError {
    match error {
        ArenaError::InsufficientStorage { required, provided } => {
            EncoderWorkspaceError::InsufficientWorkspace { required, provided }
        }
        ArenaError::CapacityExceeded { .. } | ArenaError::SizeOverflow => {
            EncoderWorkspaceError::SizeOverflow
        }
    }
}

fn checked_output_size(input_size: usize) -> Result<usize, EncoderWorkspaceError> {
    let bound = c_port::workspace_compress_bound(input_size);
    if bound == 0 {
        Err(EncoderWorkspaceError::SizeOverflow)
    } else {
        Ok(bound)
    }
}
