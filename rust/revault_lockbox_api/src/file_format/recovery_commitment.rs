//! Bounded recovery-commitment experiment; not a production archive encoding.
//!
//! A caller must authenticate the selected publication and its established owner
//! before trusting a Root. An inclusion proof alone does not prove publication,
//! freshness, correct canonical metadata, or the bytes of an unread payload.
use sha2::{Digest, Sha256};

type Hash = [u8; 32];
pub const MAX_OBJECTS: usize = 1_000_000;
const MAX_KEY: usize = 4096;
const MAX_METADATA: usize = 65536;
const MAX_TABLE_METADATA: usize = 256 * 1024 * 1024;
const PROOF_MAGIC: &[u8; 8] = b"RVP4PRF1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Limit,
    Invalid,
    Ordering,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    pub archive: [u8; 16],
    pub sequence: u64,
    pub format: u64,
}

/// Namespace separates files, variables, forms, access and allocation metadata.
/// Metadata must be a canonical descriptor containing all semantics and extents.
#[derive(Clone, Debug)]
pub struct Object {
    pub namespace: u8,
    pub key: Vec<u8>,
    pub metadata: Vec<u8>,
    pub logical_len: u64,
    pub content: Hash,
}
impl Object {
    fn validate(&self) -> Result<()> {
        if !(1..=8).contains(&self.namespace) || self.key.is_empty() {
            return Err(Error::Invalid);
        }
        if self.key.len() > MAX_KEY || self.metadata.len() > MAX_METADATA {
            return Err(Error::Limit);
        }
        Ok(())
    }
    fn hash(&self) -> Result<Hash> {
        self.validate()?;
        let mut hash = Sha256::new();
        hash.update(b"revault-experiment-object-v1\0");
        hash.update([self.namespace]);
        field(&mut hash, &self.key);
        field(&mut hash, &self.metadata);
        hash.update(self.logical_len.to_le_bytes());
        hash.update(self.content);
        Ok(hash.finalize().into())
    }
}
fn field(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}
fn validate_table(objects: &[Object]) -> Result<usize> {
    if objects.len() > MAX_OBJECTS {
        return Err(Error::Limit);
    }
    let mut bytes = 0usize;
    for (index, object) in objects.iter().enumerate() {
        object.validate()?;
        bytes = bytes
            .checked_add(object.key.len())
            .and_then(|n| n.checked_add(object.metadata.len()))
            .ok_or(Error::Limit)?;
        if bytes > MAX_TABLE_METADATA {
            return Err(Error::Limit);
        }
        if index > 0 {
            let previous = &objects[index - 1];
            if (previous.namespace, &previous.key) >= (object.namespace, &object.key) {
                return Err(Error::Ordering);
            }
        }
    }
    Ok(bytes)
}
fn parent(left: Hash, right: Hash) -> Hash {
    let mut hash = Sha256::new();
    hash.update(b"revault-experiment-node-v1\0");
    hash.update(left);
    hash.update(right);
    hash.finalize().into()
}
fn empty() -> Hash {
    Sha256::digest(b"revault-experiment-empty-v1\0").into()
}
fn seal(context: &Context, count: usize, hash: Hash, tree: bool) -> Hash {
    let mut digest = Sha256::new();
    digest.update(b"revault-experiment-root-v1\0");
    digest.update([u8::from(tree)]);
    digest.update(context.archive);
    digest.update(context.sequence.to_le_bytes());
    digest.update(context.format.to_le_bytes());
    digest.update((count as u64).to_le_bytes());
    digest.update(hash);
    digest.finalize().into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Root {
    pub context: Context,
    pub count: u32,
    pub digest: Hash,
}
impl Root {
    /// Domain-separated bytes to authenticate with the established owner key.
    pub fn message(&self) -> Vec<u8> {
        let mut out = b"revault-experiment-owner-root-v1\0".to_vec();
        out.extend_from_slice(&self.context.archive);
        out.extend_from_slice(&self.context.sequence.to_le_bytes());
        out.extend_from_slice(&self.context.format.to_le_bytes());
        out.extend_from_slice(&self.count.to_le_bytes());
        out.extend_from_slice(&self.digest);
        out
    }
    pub fn verify(&self, object: &Object, proof: &Proof) -> Result<()> {
        proof.validate()?;
        if self.count != proof.count {
            return Err(Error::Invalid);
        }
        let mut hash = object.hash()?;
        let mut index = proof.index;
        for sibling in &proof.siblings {
            hash = if index & 1 == 0 {
                parent(hash, *sibling)
            } else {
                parent(*sibling, hash)
            };
            index >>= 1;
        }
        if seal(&self.context, self.count as usize, hash, true) != self.digest {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}

pub fn flat_digest(context: &Context, objects: &[Object]) -> Result<Hash> {
    validate_table(objects)?;
    let mut hash = Sha256::new();
    hash.update(b"revault-experiment-flat-v1\0");
    for object in objects {
        hash.update(object.hash()?);
    }
    Ok(seal(context, objects.len(), hash.finalize().into(), false))
}

pub struct Tree {
    root: Root,
    levels: Vec<Vec<Hash>>,
    metadata_bytes: usize,
}
impl Tree {
    pub fn build(context: Context, objects: &[Object]) -> Result<Self> {
        let metadata_bytes = validate_table(objects)?;
        let width = objects.len().max(1).next_power_of_two();
        let mut leaves = Vec::with_capacity(width);
        for object in objects {
            leaves.push(object.hash()?);
        }
        leaves.resize(width, empty());
        let mut levels = vec![leaves];
        while levels.last().unwrap().len() > 1 {
            let next = levels
                .last()
                .unwrap()
                .chunks_exact(2)
                .map(|pair| parent(pair[0], pair[1]))
                .collect();
            levels.push(next);
        }
        let root = Root {
            digest: seal(&context, objects.len(), levels.last().unwrap()[0], true),
            context,
            count: objects.len() as u32,
        };
        Ok(Self {
            root,
            levels,
            metadata_bytes,
        })
    }
    pub fn root(&self) -> &Root {
        &self.root
    }
    pub fn retained_hash_bytes(&self) -> usize {
        self.levels.iter().map(|level| level.capacity() * 32).sum()
    }
    /// In-memory same-identity replacement; not a persisted publication protocol.
    pub fn replace(&mut self, index: usize, old: &Object, new: &Object) -> Result<()> {
        if index >= self.root.count as usize
            || old.namespace != new.namespace
            || old.key != new.key
            || self.levels[0][index] != old.hash()?
        {
            return Err(Error::Invalid);
        }
        let hash = new.hash()?;
        let bytes = self
            .metadata_bytes
            .checked_sub(old.key.len() + old.metadata.len())
            .and_then(|n| n.checked_add(new.key.len() + new.metadata.len()))
            .ok_or(Error::Limit)?;
        if bytes > MAX_TABLE_METADATA {
            return Err(Error::Limit);
        }
        self.metadata_bytes = bytes;
        self.levels[0][index] = hash;
        let mut position = index;
        for level in 1..self.levels.len() {
            position >>= 1;
            self.levels[level][position] = parent(
                self.levels[level - 1][position * 2],
                self.levels[level - 1][position * 2 + 1],
            );
        }
        self.root.digest = seal(
            &self.root.context,
            self.root.count as usize,
            self.levels.last().unwrap()[0],
            true,
        );
        Ok(())
    }
    pub fn proof(&self, index: usize) -> Result<Proof> {
        if index >= self.root.count as usize {
            return Err(Error::Invalid);
        }
        let mut position = index;
        let mut siblings = Vec::with_capacity(self.levels.len() - 1);
        for level in self.levels.iter().take(self.levels.len() - 1) {
            siblings.push(level[position ^ 1]);
            position >>= 1;
        }
        Ok(Proof {
            count: self.root.count,
            index: index as u32,
            siblings,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    count: u32,
    index: u32,
    siblings: Vec<Hash>,
}
impl Proof {
    fn depth(count: u32) -> Result<usize> {
        if count == 0 || count as usize > MAX_OBJECTS {
            return Err(Error::Limit);
        }
        Ok((count as usize).next_power_of_two().trailing_zeros() as usize)
    }
    fn validate(&self) -> Result<()> {
        if self.index >= self.count || self.siblings.len() != Self::depth(self.count)? {
            return Err(Error::Invalid);
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut out = Vec::with_capacity(24 + self.siblings.len() * 32);
        out.extend_from_slice(PROOF_MAGIC);
        out.extend_from_slice(&self.count.to_le_bytes());
        out.extend_from_slice(&self.index.to_le_bytes());
        out.push(self.siblings.len() as u8);
        out.extend_from_slice(&[0; 7]);
        for sibling in &self.siblings {
            out.extend_from_slice(sibling);
        }
        Ok(out)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 24 || &bytes[..8] != PROOF_MAGIC || bytes[17..24] != [0; 7] {
            return Err(Error::Invalid);
        }
        let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let index = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
        let depth = Self::depth(count)?;
        if index >= count || usize::from(bytes[16]) != depth || bytes.len() != 24 + 32 * depth {
            return Err(Error::Invalid);
        }
        // The untrusted count and exact length have been bounded before allocating.
        let siblings = bytes[24..]
            .chunks_exact(32)
            .map(|hash| hash.try_into().unwrap())
            .collect();
        Ok(Self {
            count,
            index,
            siblings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> Context {
        Context {
            archive: [7; 16],
            sequence: 19,
            format: 3,
        }
    }
    fn objects(count: usize) -> Vec<Object> {
        (0..count)
            .map(|index| Object {
                namespace: 1,
                key: format!("/file-{index:06}").into_bytes(),
                metadata: b"canonical file descriptor".to_vec(),
                logical_len: 5,
                content: Sha256::digest(b"hello").into(),
            })
            .collect()
    }
    #[test]
    fn independent_little_endian_vector_matches() {
        let vector: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/recovery_commitment_v1.json"
        ))
        .unwrap();
        let decode = |text: &str| {
            (0..text.len())
                .step_by(2)
                .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).unwrap())
                .collect::<Vec<_>>()
        };
        let content: Hash = decode(vector["content_sha256"].as_str().unwrap())
            .try_into()
            .unwrap();
        let objects: Vec<_> = vector["keys"]
            .as_array()
            .unwrap()
            .iter()
            .map(|key| Object {
                namespace: 1,
                key: key.as_str().unwrap().as_bytes().to_vec(),
                metadata: b"m".to_vec(),
                logical_len: 5,
                content,
            })
            .collect();
        let tree = Tree::build(context(), &objects).unwrap();
        assert_eq!(
            tree.root().digest.as_slice(),
            decode(vector["root_sha256"].as_str().unwrap())
        );
        let bytes = decode(vector["proof_hex"].as_str().unwrap());
        assert_eq!(tree.proof(1).unwrap().encode().unwrap(), bytes);
        tree.root()
            .verify(&objects[1], &Proof::decode(&bytes).unwrap())
            .unwrap();
    }
    #[test]
    fn every_position_including_odd_trees_round_trips() {
        for count in 1..=129 {
            let objects = objects(count);
            let tree = Tree::build(context(), &objects).unwrap();
            for (index, object) in objects.iter().enumerate() {
                let proof = tree.proof(index).unwrap();
                let decoded = Proof::decode(&proof.encode().unwrap()).unwrap();
                tree.root().verify(object, &decoded).unwrap();
            }
        }
        let empty = Tree::build(context(), &[]).unwrap();
        assert_eq!(empty.root().count, 0);
        assert!(empty.proof(0).is_err());
    }
    #[test]
    fn independent_proof_survives_unrelated_object_loss() {
        let mut objects = objects(9);
        let tree = Tree::build(context(), &objects).unwrap();
        let proof = tree.proof(2).unwrap();
        let original_flat = flat_digest(&context(), &objects).unwrap();
        objects[7].metadata[0] ^= 1;
        assert_ne!(flat_digest(&context(), &objects).unwrap(), original_flat);
        let survivor = objects.remove(2);
        drop(objects);
        drop(tree.levels);
        tree.root.verify(&survivor, &proof).unwrap();
    }
    #[test]
    fn all_object_and_context_fields_and_proof_positions_are_bound() {
        let objects = objects(3);
        let tree = Tree::build(context(), &objects).unwrap();
        let proof = tree.proof(0).unwrap();
        for which in 0..5 {
            let mut object = objects[0].clone();
            match which {
                0 => object.namespace = 2,
                1 => object.key.push(b'x'),
                2 => object.metadata.push(0),
                3 => object.logical_len += 1,
                _ => object.content[0] ^= 1,
            }
            assert!(tree.root().verify(&object, &proof).is_err());
        }
        for which in 0..4 {
            let mut root = tree.root().clone();
            match which {
                0 => root.context.archive[0] ^= 1,
                1 => root.context.sequence += 1,
                2 => root.context.format ^= 1,
                _ => root.count += 1,
            }
            assert!(root.verify(&objects[0], &proof).is_err());
        }
        for object in objects.iter().skip(1) {
            assert!(tree.root().verify(object, &proof).is_err());
        }
        let mut damaged = proof;
        damaged.siblings[0][0] ^= 1;
        assert!(tree.root().verify(&objects[0], &damaged).is_err());
    }
    #[test]
    fn stale_deleted_and_unpublished_membership_do_not_match_selected_root() {
        let old_objects = objects(3);
        let old = Tree::build(context(), &old_objects).unwrap();
        let old_proof = old.proof(1).unwrap();
        let mut newer = context();
        newer.sequence += 1;
        let committed =
            Tree::build(newer, &[old_objects[0].clone(), old_objects[2].clone()]).unwrap();
        assert!(committed
            .root()
            .verify(&old_objects[1], &old_proof)
            .is_err());
        let mut future = context();
        future.sequence += 2;
        let unpublished = Tree::build(future, &old_objects).unwrap();
        assert!(committed
            .root()
            .verify(&old_objects[1], &unpublished.proof(1).unwrap())
            .is_err());
        // Even unchanged membership from another generation needs the selected root.
        assert_ne!(old.root().message(), unpublished.root().message());
    }
    #[test]
    fn replacements_match_full_rebuild_and_reject_identity_changes() {
        let mut objects = objects(33);
        let mut tree = Tree::build(context(), &objects).unwrap();
        for index in 0..objects.len() {
            let old = objects[index].clone();
            objects[index].metadata.push(index as u8);
            tree.replace(index, &old, &objects[index]).unwrap();
            assert_eq!(
                tree.root(),
                Tree::build(context(), &objects).unwrap().root()
            );
            tree.root()
                .verify(&objects[index], &tree.proof(index).unwrap())
                .unwrap();
            assert!(tree.replace(index, &old, &objects[index]).is_err());
        }
        let original = tree.root().clone();
        let mut renamed = objects[0].clone();
        renamed.key.push(b'x');
        assert!(tree.replace(0, &objects[0], &renamed).is_err());
        assert_eq!(tree.root(), &original);
    }
    #[test]
    fn malformed_proofs_and_ambiguous_tables_are_rejected() {
        let objects = objects(3);
        let tree = Tree::build(context(), &objects).unwrap();
        let bytes = tree.proof(1).unwrap().encode().unwrap();
        for end in 0..bytes.len() {
            assert!(Proof::decode(&bytes[..end]).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Proof::decode(&trailing).is_err());
        for offset in [0, 8, 12, 16, 17, 23] {
            let mut corrupt = bytes.clone();
            corrupt[offset] = 255;
            assert!(Proof::decode(&corrupt).is_err(), "offset {offset}");
        }
        let mut oversize = bytes;
        oversize[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(Proof::decode(&oversize), Err(Error::Limit));
        assert!(Tree::build(context(), &[objects[0].clone(), objects[0].clone()]).is_err());
        assert!(Tree::build(context(), &[objects[1].clone(), objects[0].clone()]).is_err());
        let mut excessive = objects[0].clone();
        excessive.metadata.resize(MAX_METADATA + 1, 0);
        assert!(Tree::build(context(), &[excessive]).is_err());
    }
}
