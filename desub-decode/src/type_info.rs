
pub trait TypeInfo {
    type TypeId;
    fn get_type_info(&'_ self, type_id: &Self::TypeId) -> Option<TypeMarker<'_>>;
}

pub enum TypeMarker<'a> {

}