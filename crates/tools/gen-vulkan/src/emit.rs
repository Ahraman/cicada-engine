use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Visibility};

use crate::{
    error::{EmitError, Error},
    repr::{Decor, StructMember, StructType, TypeInfo, TypeKind, Vulkan},
};

pub struct EmitSettings {
    pub output_dir: PathBuf,
}

impl Vulkan {
    pub fn emit(self, settings: EmitSettings) -> Result<(), Error> {
        self.emit_root_module(&settings)?;
        if self.has_commands() {
            self.emit_cmds_module(&settings)?;
        }
        if self.has_enums() {
            self.emit_enums_module(&settings)?;
        }
        if self.has_result() {
            self.emit_result_module(&settings)?;
        }
        if self.has_structs() {
            self.emit_structs_module(&settings)?;
        }
        if self.has_unions() {
            self.emit_unions_module(&settings)?;
        }

        Ok(())
    }

    fn emit_root_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("mod.rs"))?);
        if let Some(content) = self.root_module_head()? {
            writer.write(content.as_bytes())?;
        }

        writer.flush()?;
        Ok(())
    }

    fn root_module_head(&self) -> Result<Option<String>, EmitError> {
        let mut submodules = Vec::new();
        if self.has_commands() {
            submodules.push("commands");
        }
        if self.has_enums() {
            submodules.push("enums");
        }
        if self.has_result() {
            submodules.push("result");
        }
        if self.has_structs() {
            submodules.push("structs");
        }
        if self.has_unions() {
            submodules.push("unions");
        }

        let submodules = submodules;
        Ok(if !submodules.is_empty() {
            let mod_defs = submodules
                .iter()
                .fold(TokenStream::default(), |acc, submodule| {
                    let mod_name = Ident::new(submodule, Span::call_site());
                    quote! {
                        #acc
                        mod #mod_name;
                    }
                });

            let mod_uses = submodules
                .iter()
                .fold(TokenStream::default(), |acc, submodule| {
                    let mod_name = Ident::new(submodule, Span::call_site());
                    quote! {
                        #acc #mod_name::*,
                    }
                });

            let mod_uses = quote! {
                pub use self::{#mod_uses};
            };

            let tokens = quote! {
                #mod_defs

                #mod_uses
            };

            Some(prettyplease::unparse(&syn::parse2(tokens)?))
        } else {
            None
        })
    }

    fn emit_cmds_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("cmds.rs"))?);

        writer.flush()?;
        Ok(())
    }

    fn emit_enums_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("enums.rs"))?);

        writer.flush()?;
        Ok(())
    }

    fn emit_result_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("result.rs"))?);

        writer.flush()?;
        Ok(())
    }

    fn emit_structs_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("structs.rs"))?);
        self.emit_structs_module_body(&mut writer)?;

        writer.flush()?;
        Ok(())
    }

    fn emit_structs_module_body(&self, writer: &mut impl Write) -> Result<(), Error> {
        for (type_info, struct_type) in self.types.items.iter().filter_map(|(_, ty)| {
            if let TypeKind::Struct(struct_type) = &ty.kind {
                Some((&ty.info, struct_type))
            } else {
                None
            }
        }) {
            writer.write(struct_type.emit_defn(self, type_info)?.as_bytes())?;
            self.write_struct_impls(writer, type_info, struct_type)?;
        }

        Ok(())
    }

    fn write_struct_impls(
        &self,
        writer: &mut impl Write,
        type_info: &TypeInfo,
        struct_type: &StructType,
    ) -> Result<(), Error> {
        if let Some(content) = struct_type.emit_impl_default(self, type_info)? {
            writer.write(content.as_bytes())?;
        }

        Ok(())
    }

    fn emit_unions_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("unions.rs"))?);
        writer.flush()?;

        Ok(())
    }
}

impl TypeInfo {
    fn decorated_name(&self, decors: &[Decor]) -> TokenStream {
        let name = Ident::new(&self.output_name, Span::call_site());
        let mut res = quote! {
            #name
        };

        for decor in decors.iter().rev() {
            res = match decor {
                Decor::Const => res,
                Decor::ConstPtr => quote! { *const #res },
                Decor::MutPtr => quote! { *mut #res },
            }
        }

        res
    }

    fn screaming_name(&self) -> String {
        self.output_name
            .split(|ch: char| ch.is_ascii_uppercase())
            .map(|s| s.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join("_")
    }

    fn output_feature(&self, vk: &Vulkan) -> Option<TokenStream> {
        if let Some(handle) = self.feature {
            if let Some(feature) = vk.features.get(handle) {
                let feature_name = feature.header.output_name.as_str();
                Some(quote! {
                    #[cfg(feature = #feature_name)]
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn default_value(&self, decors: &[Decor]) -> TokenStream {
        match decors.iter().rev().next() {
            Some(Decor::ConstPtr) => quote! { std::ptr::null() },
            Some(Decor::MutPtr) => quote! { std::ptr::null_mut() },
            _ => {
                if self.standard_name == "VkStructureType" {
                    let default_value = Ident::new(&self.screaming_name(), Span::call_site());
                    quote! {
                        vk::StructureType::#default_value
                    }
                } else {
                    quote! {
                        Default::default()
                    }
                }
            }
        }
    }
}

impl StructType {
    fn emit_defn(&self, vk: &Vulkan, type_info: &TypeInfo) -> Result<String, EmitError> {
        let member_defns = self.members.iter().fold(
            Ok(TokenStream::new()),
            |acc: Result<_, EmitError>, member| {
                let member_defn = member.emit_defn(vk, type_info)?;
                acc.map(|acc| {
                    quote! {
                        #acc
                        #member_defn,
                    }
                })
            },
        )?;

        let feature_line = type_info.output_feature(vk).unwrap_or_default();
        let struct_name = Ident::new(&type_info.output_name, Span::call_site());
        let visibility = Visibility::Public(Default::default());
        let tokens = quote! {
            #feature_line
            #[repr(C)]
            #visibility struct #struct_name {
                #member_defns
            }
        };

        Ok(prettyplease::unparse(&syn::parse2(tokens)?))
    }

    fn emit_impl_default(
        &self,
        vk: &Vulkan,
        type_info: &TypeInfo,
    ) -> Result<Option<String>, EmitError> {
        let type_name = type_info.output_name.as_str();
        let feature_line = type_info.output_feature(vk).unwrap_or_default();

        let member_defaults = self.members.iter().fold(
            Ok(TokenStream::new()),
            |acc: Result<_, EmitError>, member| {
                let member_default = member.emit_default(vk, type_info)?;
                acc.map(|acc| {
                    quote! {
                        #acc
                        #member_default,
                    }
                })
            },
        )?;

        let tokens = quote! {
            #feature_line
            impl Default for #type_name {
                fn default() -> Self {
                    Self {
                        #member_defaults
                    }
                }
            }
        };

        Ok(Some(prettyplease::unparse(&syn::parse2(tokens)?)))
    }
}

impl StructMember {
    fn emit_default(&self, vk: &Vulkan, type_info: &TypeInfo) -> Result<TokenStream, EmitError> {
        let member_name = Ident::new(&self.output_name, Span::call_site());
        let member_type =
            vk.types
                .get(self.decor_type.handle)
                .ok_or(EmitError::BadStructMember(
                    type_info.standard_name.clone(),
                    self.standard_name.clone(),
                ))?;

        let default_value = member_type.info.default_value(&self.decor_type.decors);
        Ok(quote! {
            #member_name: #default_value,
        })
    }

    fn emit_defn(&self, vk: &Vulkan, type_info: &TypeInfo) -> Result<TokenStream, EmitError> {
        let member_name = Ident::new(&self.output_name, Span::call_site());
        let member_type =
            vk.types
                .get(self.decor_type.handle)
                .ok_or(EmitError::BadStructMember(
                    type_info.standard_name.clone(),
                    self.standard_name.clone(),
                ))?;

        let type_name = member_type.info.decorated_name(&self.decor_type.decors);
        let visibility = Visibility::Public(Default::default());
        Ok(quote! {
            #visibility #member_name: #type_name,
        })
    }
}
