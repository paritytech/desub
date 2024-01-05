The plan
========

In `scale-decode`:

- Modify `Visitor` trait to have associated types `TypeId` and `TypeInfo: TypeInfoGetter<TypeId=Self::TypeId>`.
- `pub trait TypeInfoGetter { fn get_type_info(&self, type_id: &Self::TypeId) -> Option<TypeInfo> }`
- `pub enum TypeInfo<'a, TypeId> { SequenceOf(Cow<'a, TypeId>), ArrayOf(Cow<'a, TypeId>, usize), ... }`
- impl this for `PortableRegistry`.
- Fix everything that this impacts. Eg all visitor types will need extra generics
- Fix decoding to work with more generic visitor type and TypeInfo
- Can we have a more generic `DecodeAsType` and more generic `Visitor` impls on builtin types? `Intovisitor` may also need to become more generic for this to work.

When this is all done:

- Fix `scale-value` and upstream crates to be happy with this new `scale-decode`
- Then, `desub-core` could perhaps be focused on providing traits for decoding extrinsics and storage items (with functionality for doing so just like `scale-decode` has functionality for decoding stuff),
  and impls behind legacy/current feature flags, and `desub` could build on these impls and provide a nice interface to enable decoding stuff given metadata and, where needed, type mappings.
- Then, have subxt rely on `desub-core` to avoid it duplicating logic for decoding extrinsics and storage items.