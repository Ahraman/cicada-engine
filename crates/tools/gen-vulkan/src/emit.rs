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
    repr::{StructMember, StructType, TypeInfo, TypeKind, Vulkan},
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
            let mut mod_defs = TokenStream::new();
            for submodule in submodules.iter() {
                let mod_name = Ident::new(*submodule, Span::call_site());
                let mod_def = quote! {
                    mod #mod_name;
                };

                mod_defs.extend(mod_def);
            }

            let mut mod_uses = TokenStream::new();
            for submodule in submodules.iter() {
                let mod_name = Ident::new(*submodule, Span::call_site());
                let mod_use = quote! {
                    #mod_name::*,
                };

                mod_uses.extend(mod_use);
            }

            let mod_uses = quote! {
                pub use self::{#mod_uses};
            };

            let head = quote! {
                #mod_defs

                #mod_uses
            };

            Some(prettyplease::unparse(&syn::parse2(head)?))
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
            writer.write(self.struct_defn(type_info, struct_type)?.as_bytes())?;
        }

        Ok(())
    }

    fn struct_defn(
        &self,
        type_info: &TypeInfo,
        struct_type: &StructType,
    ) -> Result<String, EmitError> {
        let struct_name = Ident::new(&type_info.output_name, Span::call_site());

        let mut members_defn = TokenStream::new();
        for member in struct_type.members.iter() {
            let member_defn = self.struct_member_defn(type_info, member)?;
            members_defn = quote! {
                #members_defn
                #member_defn,
            };
        }

        let feature_line = if let Some(handle) = type_info.feature {
            if let Some(feature) = self.features.get(handle) {
                let feature_name = feature.header.output_name.as_str();
                quote! {
                    #[cfg(feature = #feature_name)]
                }
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

        let visibility = Visibility::Public(Default::default());
        let struct_defn = quote! {
            #feature_line
            #visibility struct #struct_name {
                #members_defn
            }
        };

        Ok(prettyplease::unparse(&syn::parse2(struct_defn)?))
    }

    fn struct_member_defn(
        &self,
        type_info: &TypeInfo,
        member: &StructMember,
    ) -> Result<TokenStream, EmitError> {
        let member_name = Ident::new(&member.output_name, Span::call_site());
        let member_type =
            self.types
                .get(member.decor_type.handle)
                .ok_or(EmitError::BadStructMember(
                    type_info.standard_name.clone(),
                    member.standard_name.clone(),
                ))?;

        let type_name = member_type.decorated_name(&member.decor_type.decors);
        let visibility = Visibility::Public(Default::default());
        Ok(quote! {
            #visibility #member_name: #type_name,
        })
    }

    fn emit_unions_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("unions.rs"))?);
        writer.flush()?;

        Ok(())
    }
}
