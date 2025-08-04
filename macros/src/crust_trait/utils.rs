use syn::{GenericParam, Ident, Type, TypeParamBound};

pub fn map_method_ident(ident: Ident) -> Ident {
    Ident::new(&format!("{}METHOD", ident), ident.span())
}

pub fn map_genrics_ident(param: &mut GenericParam, mapper: &dyn Fn(Ident) -> Ident) {
    match param {
        GenericParam::Lifetime(lifetime_param) => {
            lifetime_param.lifetime.ident = mapper(lifetime_param.lifetime.ident.clone());
            lifetime_param.bounds.iter_mut().for_each(|lifetime| {
                lifetime.ident = mapper(lifetime.ident.clone());
            });
        }
        GenericParam::Type(type_param) => {
            type_param.ident = mapper(type_param.ident.clone());
            type_param.bounds.iter_mut().for_each(|bound| {
                if let TypeParamBound::Lifetime(lifetime) = bound {
                    lifetime.ident = mapper(lifetime.ident.clone());
                }
            });
        }
        GenericParam::Const(const_param) => {
            const_param.ident = mapper(const_param.ident.clone());
        }
    }
}

pub fn map_ty(ty: &mut Type, mapper: &dyn Fn(&mut syn::TypePath)) {
    match ty {
        Type::Reference(type_ref) => {
            map_ty(&mut type_ref.elem, &mapper);
        }
        Type::Array(type_array) => {
            map_ty(&mut type_array.elem, &mapper);
        }
        Type::BareFn(type_bare_fn) => {
            type_bare_fn.inputs.iter_mut().for_each(|i| {
                map_ty(&mut i.ty, &mapper);
            });
        }
        Type::Group(type_group) => {
            map_ty(&mut type_group.elem, &mapper);
        }
        Type::Paren(type_paren) => {
            map_ty(&mut type_paren.elem, &mapper);
        }
        Type::Path(type_path) => {
            mapper(type_path);
        }
        Type::Ptr(type_ptr) => {
            map_ty(&mut type_ptr.elem, &mapper);
        }
        Type::Slice(type_slice) => {
            map_ty(&mut type_slice.elem, &mapper);
        }

        Type::Tuple(type_tuple) => {
            type_tuple.elems.iter_mut().for_each(|i| {
                map_ty(i, &mapper);
            });
        }

        _ => {}
    };
}
