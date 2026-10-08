//! Small authenticated form records. Full user texts live only in guarded pages.
use super::*;
use crate::file_format::{
    authenticated_index::Entry,
    form_segments::{self, Layout},
};
use crate::{
    FormDefinition, FormFieldDefinition, FormFieldKind, FormFieldValue, FormRecord, FormTypeId,
    FormValue, LockboxPath, SecretString, VariableSensitivity,
};
use std::{collections::BTreeMap, sync::Arc};
const DEFINITION: u8 = 7;
const DEFINITION_FIELD: u8 = 8;
const RECORD: u8 = 9;
const CAPTURE: u8 = 10;
const MAX_ROWS: usize = 4096;
#[derive(Default)]
pub(in crate::file_format::candidate_files) struct Forms {
    pub definitions: Vec<Definition>,
    pub records: Vec<Record>,
}
pub(in crate::file_format::candidate_files) struct Definition {
    pub type_id: FormTypeId,
    pub revision: u32,
    pub alias: String,
    pub name: Layout,
    pub description: Layout,
    pub fields: Vec<Field>,
}
pub(in crate::file_format::candidate_files) struct Field {
    pub id: String,
    pub kind: FormFieldKind,
    pub required: bool,
    pub label: Layout,
}
pub(in crate::file_format::candidate_files) struct Record {
    pub path: LockboxPath,
    pub id: [u8; 16],
    pub type_id: FormTypeId,
    pub revision: u32,
    pub alias: String,
    pub name: Layout,
    pub fields: Vec<Capture>,
}
pub(in crate::file_format::candidate_files) struct Capture {
    pub id: String,
    pub kind: FormFieldKind,
    pub label: Layout,
    pub value: Layout,
}
pub(in crate::file_format::candidate_files) fn definition_key(
    id: &FormTypeId,
    revision: u32,
) -> Vec<u8> {
    let mut key = id.as_str().as_bytes().to_vec();
    key.extend_from_slice(&revision.to_be_bytes());
    key
}
impl Forms {
    pub fn admit(&self, additional: usize) -> Result<()> {
        let mut rows = additional;
        for n in std::iter::once(self.definitions.len())
            .chain(std::iter::once(self.records.len()))
            .chain(self.definitions.iter().map(|d| d.fields.len()))
            .chain(self.records.iter().map(|r| r.fields.len()))
        {
            rows = rows.checked_add(n).ok_or(Error::CorruptRecord)?;
        }
        if rows > MAX_ROWS {
            return Err(Error::SecurityLimitExceeded("typed form row budget".into()));
        }
        Ok(())
    }
    pub fn admit_row(entry: &Entry) -> Result<()> {
        // Alias/identifier strings use2-byte lengths and at most128 bytes; a form
        // layout uses88+16*48 bytes plus its2-byte length. Reject before accumulation.
        let (key, max_value) = match entry.namespace {
            7 => (40, 1850),
            8 => (44, 990),
            9 => (0, 1048),
            10 => (20, 1847),
            _ => return Err(Error::CorruptRecord),
        };
        if entry.value.len() > max_value
            || (key != 0 && entry.key.len() != key)
            || (key == 0
                && (entry.key.is_empty() || entry.key.len() > crate::constants::MAX_PATH_BYTES))
        {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty() && self.records.is_empty()
    }
    pub fn texts(&self) -> Vec<&Layout> {
        let mut out = Vec::new();
        for d in &self.definitions {
            out.extend([&d.name, &d.description]);
            out.extend(d.fields.iter().map(|f| &f.label));
        }
        for r in &self.records {
            out.push(&r.name);
            for f in &r.fields {
                out.extend([&f.label, &f.value]);
            }
        }
        out
    }
    pub fn texts_mut(&mut self) -> Vec<&mut Layout> {
        let mut out = Vec::new();
        for d in &mut self.definitions {
            out.extend([&mut d.name, &mut d.description]);
            out.extend(d.fields.iter_mut().map(|f| &mut f.label));
        }
        for r in &mut self.records {
            out.push(&mut r.name);
            for f in &mut r.fields {
                out.extend([&mut f.label, &mut f.value]);
            }
        }
        out
    }
    pub fn validate(&self) -> Result<()> {
        self.admit(0)?;
        let mut defs = BTreeMap::new();
        let mut record_ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        let mut text_ids = BTreeSet::new();
        let normal = VariableSensitivity::Normal;
        let text =
            |layout: &Layout, context: [u8; 32], sensitivity: VariableSensitivity| -> Result<()> {
                if layout.context != context || layout.sensitivity != sensitivity {
                    return Err(Error::CorruptRecord);
                }
                Ok(())
            };
        for d in &self.definitions {
            if d.revision == 0
                || FormTypeId::new(d.type_id.as_str())? != d.type_id
                || FormDefinition::validated_alias(&d.alias)? != d.alias
                || d.fields.is_empty()
            {
                return Err(Error::CorruptRecord);
            }
            let key = definition_key(&d.type_id, d.revision);
            if defs.insert(key.clone(), d).is_some() {
                return Err(Error::CorruptRecord);
            }
            text(&d.name, form_segments::context(&key, 1, None), normal)?;
            text(
                &d.description,
                form_segments::context(&key, 2, None),
                normal,
            )?;
            let mut ids = BTreeSet::new();
            for f in &d.fields {
                if FormFieldDefinition::validated_id(&f.id)? != f.id || !ids.insert(&f.id) {
                    return Err(Error::CorruptRecord);
                }
                text(
                    &f.label,
                    form_segments::context(&key, 3, Some((&f.id, f.kind))),
                    normal,
                )?;
            }
        }
        for r in &self.records {
            if r.id == [0; 16]
                || !record_ids.insert(r.id)
                || !paths.insert(r.path.as_str())
                || r.path.file_path()?.as_str() != r.path.as_str()
            {
                return Err(Error::CorruptRecord);
            }
            let d = defs
                .get(&definition_key(&r.type_id, r.revision))
                .ok_or(Error::CorruptRecord)?;
            if d.alias != r.alias {
                return Err(Error::CorruptRecord);
            }
            text(&r.name, form_segments::context(&r.id, 4, None), normal)?;
            let mut ids = BTreeSet::new();
            for f in &r.fields {
                if FormFieldDefinition::validated_id(&f.id)? != f.id || !ids.insert(&f.id) {
                    return Err(Error::CorruptRecord);
                }
                text(
                    &f.label,
                    form_segments::context(&r.id, 5, Some((&f.id, f.kind))),
                    normal,
                )?;
                text(
                    &f.value,
                    form_segments::context(&r.id, 6, Some((&f.id, f.kind))),
                    if f.kind.is_secret() {
                        VariableSensitivity::Secret
                    } else {
                        normal
                    },
                )?;
            }
        }
        for layout in self.texts() {
            if !text_ids.insert(layout.id) || record_ids.contains(&layout.id) {
                return Err(Error::CorruptRecord);
            }
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<Entry>> {
        self.validate()?;
        let mut rows = Vec::new();
        for d in &self.definitions {
            let key = definition_key(&d.type_id, d.revision);
            let mut w = Out::new();
            w.string(&d.alias)?;
            w.layout(&d.name)?;
            w.layout(&d.description)?;
            w.count(d.fields.len())?;
            rows.push(Entry::new(DEFINITION, &key, &w.0)?);
            for (n, f) in d.fields.iter().enumerate() {
                let mut k = key.clone();
                k.extend_from_slice(&(n as u32).to_be_bytes());
                let mut w = Out::new();
                w.string(&f.id)?;
                w.0.extend([f.kind.code(), u8::from(f.required)]);
                w.layout(&f.label)?;
                rows.push(Entry::new(DEFINITION_FIELD, &k, &w.0)?);
            }
        }
        for r in &self.records {
            let mut w = Out::new();
            w.0.extend(r.id);
            w.0.extend(definition_key(&r.type_id, r.revision));
            w.string(&r.alias)?;
            w.layout(&r.name)?;
            w.count(r.fields.len())?;
            rows.push(Entry::new(RECORD, r.path.as_str().as_bytes(), &w.0)?);
            for (n, f) in r.fields.iter().enumerate() {
                let mut k = r.id.to_vec();
                k.extend_from_slice(&(n as u32).to_be_bytes());
                let mut w = Out::new();
                w.string(&f.id)?;
                w.0.push(f.kind.code());
                w.layout(&f.label)?;
                w.layout(&f.value)?;
                rows.push(Entry::new(CAPTURE, &k, &w.0)?);
            }
        }
        Ok(rows)
    }
    pub fn decode(rows: Vec<Entry>, mode: FormatMode, sealed: u64) -> Result<Self> {
        if rows.len() > MAX_ROWS {
            return Err(Error::CorruptRecord);
        }
        let mut map = BTreeMap::new();
        for e in rows {
            Self::admit_row(&e)?;
            if map.insert((e.namespace, e.key.to_vec()), e.value).is_some() {
                return Err(Error::CorruptRecord);
            }
        }
        let mut out = Self::default();
        let defkeys: Vec<_> = map
            .keys()
            .filter(|(n, _)| *n == DEFINITION)
            .cloned()
            .collect();
        for key in defkeys {
            let (type_id, revision) = parse_definition_key(&key.1)?;
            let bytes = map.remove(&key).unwrap();
            let mut c = Input(&bytes);
            let alias = c.string()?;
            let name = c.layout(mode, sealed)?;
            let description = c.layout(mode, sealed)?;
            let count = c.count()?;
            c.end()?;
            let mut fields = Vec::new();
            for n in 0..count {
                let mut k = key.1.clone();
                k.extend_from_slice(&(n as u32).to_be_bytes());
                let bytes = map
                    .remove(&(DEFINITION_FIELD, k))
                    .ok_or(Error::CorruptRecord)?;
                let mut c = Input(&bytes);
                let id = c.string()?;
                let kind = FormFieldKind::from_code(c.byte()?)?;
                let required = match c.byte()? {
                    0 => false,
                    1 => true,
                    _ => return Err(Error::CorruptRecord),
                };
                let label = c.layout(mode, sealed)?;
                c.end()?;
                fields.push(Field {
                    id,
                    kind,
                    required,
                    label,
                });
            }
            out.definitions.push(Definition {
                type_id,
                revision,
                alias,
                name,
                description,
                fields,
            });
        }
        let keys: Vec<_> = map.keys().filter(|(n, _)| *n == RECORD).cloned().collect();
        for key in keys {
            let pathstr = std::str::from_utf8(&key.1).map_err(|_| Error::CorruptRecord)?;
            let path = LockboxPath::new(pathstr)?;
            if path.as_str() != pathstr {
                return Err(Error::CorruptRecord);
            }
            let bytes = map.remove(&key).unwrap();
            let mut c = Input(&bytes);
            let id = c.take(16)?.try_into().unwrap();
            let (type_id, revision) = parse_definition_key(c.take(40)?)?;
            let alias = c.string()?;
            let name = c.layout(mode, sealed)?;
            let count = c.count()?;
            c.end()?;
            let mut fields = Vec::new();
            for n in 0..count {
                let mut k = Vec::from(id);
                k.extend_from_slice(&(n as u32).to_be_bytes());
                let bytes = map.remove(&(CAPTURE, k)).ok_or(Error::CorruptRecord)?;
                let mut c = Input(&bytes);
                let id = c.string()?;
                let kind = FormFieldKind::from_code(c.byte()?)?;
                let label = c.layout(mode, sealed)?;
                let value = c.layout(mode, sealed)?;
                c.end()?;
                fields.push(Capture {
                    id,
                    kind,
                    label,
                    value,
                });
            }
            out.records.push(Record {
                path,
                id,
                type_id,
                revision,
                alias,
                name,
                fields,
            });
        }
        if !map.is_empty() {
            return Err(Error::CorruptRecord);
        }
        out.validate()?;
        Ok(out)
    }
    pub fn verify(
        &self,
        storage: &impl Storage,
        archive: LockboxId,
        key: &[u8; 32],
        sealed: u64,
    ) -> Result<()> {
        for d in &self.definitions {
            FormDefinition::validated_name(&read_normal(&d.name, storage, archive, key, sealed)?)?;
            FormDefinition::validated_description(&read_normal(
                &d.description,
                storage,
                archive,
                key,
                sealed,
            )?)?;
            for f in &d.fields {
                FormFieldDefinition::validated_label(&read_normal(
                    &f.label, storage, archive, key, sealed,
                )?)?;
            }
        }
        for r in &self.records {
            FormRecord::validated_name(&read_normal(&r.name, storage, archive, key, sealed)?)?;
            for f in &r.fields {
                FormFieldDefinition::validated_label(&read_normal(
                    &f.label, storage, archive, key, sealed,
                )?)?;
                let value = read_value(&f.value, storage, archive, key, sealed)?;
                f.kind.validate_value(&value)?;
            }
        }
        Ok(())
    }
}
impl Definition {
    pub fn matches(
        &self,
        expected: &FormDefinition,
        storage: &impl Storage,
        archive: LockboxId,
        key: &[u8; 32],
        sealed: u64,
    ) -> Result<bool> {
        if self.type_id != expected.type_id
            || self.revision != expected.revision
            || self.alias != expected.alias
            || self.fields.len() != expected.fields.len()
        {
            return Ok(false);
        }
        let same = |layout: &Layout, text: &str| -> Result<bool> {
            if layout.length != text.len() {
                return Ok(false);
            }
            let value = layout.read(storage, archive, key, sealed)?;
            Ok(value.with_bytes(|bytes| bytes == text.as_bytes())?)
        };
        if !same(&self.name, &expected.name)? || !same(&self.description, &expected.description)? {
            return Ok(false);
        }
        for (field, expected) in self.fields.iter().zip(&expected.fields) {
            if field.id != expected.id
                || field.kind != expected.kind
                || field.required != expected.required
                || !same(&field.label, &expected.label)?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub fn read(
        &self,
        storage: &impl Storage,
        archive: LockboxId,
        key: &[u8; 32],
        sealed: u64,
    ) -> Result<FormDefinition> {
        let fields = self
            .fields
            .iter()
            .map(|f| {
                Ok(FormFieldDefinition {
                    id: f.id.clone(),
                    label: read_normal(&f.label, storage, archive, key, sealed)?,
                    kind: f.kind,
                    required: f.required,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        FormDefinition::validated(
            self.type_id.clone(),
            self.alias.clone(),
            self.revision,
            &read_normal(&self.name, storage, archive, key, sealed)?,
            &read_normal(&self.description, storage, archive, key, sealed)?,
            fields,
        )
    }
}
impl Record {
    pub fn read(
        &self,
        storage: &impl Storage,
        archive: LockboxId,
        key: &[u8; 32],
        sealed: u64,
    ) -> Result<FormRecord> {
        let mut values = Vec::new();
        for f in &self.fields {
            let value = read_value(&f.value, storage, archive, key, sealed)?;
            f.kind.validate_value(&value)?;
            values.push(FormFieldValue {
                field_id: f.id.clone(),
                captured_label: FormFieldDefinition::validated_label(&read_normal(
                    &f.label, storage, archive, key, sealed,
                )?)?,
                kind: f.kind,
                value,
            });
        }
        Ok(FormRecord {
            path: self.path.clone(),
            name: FormRecord::validated_name(&read_normal(
                &self.name, storage, archive, key, sealed,
            )?)?,
            type_id: self.type_id.clone(),
            definition_alias: self.alias.clone(),
            definition_revision: self.revision,
            values,
        })
    }
}
fn read_normal(
    layout: &Layout,
    storage: &impl Storage,
    archive: LockboxId,
    key: &[u8; 32],
    sealed: u64,
) -> Result<String> {
    if layout.sensitivity != VariableSensitivity::Normal {
        return Err(Error::CorruptRecord);
    }
    let value = layout.read(storage, archive, key, sealed)?;
    value.with_bytes(|bytes| {
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| Error::CorruptRecord)
    })?
}
fn read_value(
    layout: &Layout,
    storage: &impl Storage,
    archive: LockboxId,
    key: &[u8; 32],
    sealed: u64,
) -> Result<FormValue> {
    if layout.sensitivity == VariableSensitivity::Secret {
        Ok(FormValue::Secret(Arc::new(SecretString::from_secure_vec(
            layout.read(storage, archive, key, sealed)?,
        ))))
    } else {
        Ok(FormValue::Normal(read_normal(
            layout, storage, archive, key, sealed,
        )?))
    }
}
fn parse_definition_key(bytes: &[u8]) -> Result<(FormTypeId, u32)> {
    if bytes.len() != 40 {
        return Err(Error::CorruptRecord);
    }
    let text = std::str::from_utf8(&bytes[..36]).map_err(|_| Error::CorruptRecord)?;
    let id = FormTypeId::new(text)?;
    let revision = u32::from_be_bytes(bytes[36..40].try_into().unwrap());
    if id.as_str() != text || revision == 0 {
        return Err(Error::CorruptRecord);
    }
    Ok((id, revision))
}
struct Out(Vec<u8>);
impl Out {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn count(&mut self, n: usize) -> Result<()> {
        if n > MAX_ROWS {
            return Err(Error::SecurityLimitExceeded(
                "typed form field budget".into(),
            ));
        }
        self.0.extend_from_slice(&(n as u32).to_le_bytes());
        Ok(())
    }
    fn string(&mut self, s: &str) -> Result<()> {
        if s.len() > 128 {
            return Err(Error::CorruptRecord);
        }
        self.0.extend_from_slice(&(s.len() as u16).to_le_bytes());
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
    fn layout(&mut self, l: &Layout) -> Result<()> {
        let b = l.encode_metadata();
        if b.len() > 856 {
            return Err(Error::CorruptRecord);
        }
        self.0.extend_from_slice(&(b.len() as u16).to_le_bytes());
        self.0.extend(b);
        Ok(())
    }
}
struct Input<'a>(&'a [u8]);
impl<'a> Input<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.0.len() {
            return Err(Error::CorruptRecord);
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn len(&mut self) -> Result<usize> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()) as usize)
    }
    fn string(&mut self) -> Result<String> {
        let n = self.len()?;
        if n > 128 {
            return Err(Error::CorruptRecord);
        }
        Ok(std::str::from_utf8(self.take(n)?)
            .map_err(|_| Error::CorruptRecord)?
            .to_owned())
    }
    fn layout(&mut self, m: FormatMode, s: u64) -> Result<Layout> {
        let n = self.len()?;
        if n > 856 {
            return Err(Error::CorruptRecord);
        }
        Layout::decode_metadata(self.take(n)?, m, s)
    }
    fn count(&mut self) -> Result<usize> {
        let n = u32::from_le_bytes(self.take(4)?.try_into().unwrap()) as usize;
        if n > MAX_ROWS {
            return Err(Error::CorruptRecord);
        }
        Ok(n)
    }
    fn end(self) -> Result<()> {
        if !self.0.is_empty() {
            return Err(Error::CorruptRecord);
        }
        Ok(())
    }
}

impl Capture {
    pub fn read_value(
        &self,
        storage: &impl Storage,
        archive: LockboxId,
        key: &[u8; 32],
        sealed: u64,
    ) -> Result<FormValue> {
        let value = read_value(&self.value, storage, archive, key, sealed)?;
        self.kind.validate_value(&value)?;
        Ok(value)
    }
}
