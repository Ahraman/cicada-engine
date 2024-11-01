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
        if self.metadata.has_commands {
            self.emit_cmds_module(&settings)?;
        }
        if self.metadata.has_enum_types {
            self.emit_enums_module(&settings)?;
        }
        if self.metadata.has_structs {
            self.emit_structs_module(&settings)?;
        }

        Ok(())
    }

    fn emit_root_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("mod.rs"))?);
        writer.write(self.root_module_content()?.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    fn root_module_content(&self) -> Result<String, EmitError> {
        let mut submodules = Vec::new();
        if self.metadata.has_commands {
            submodules.push("commands");
        }
        if self.metadata.has_enum_types {
            submodules.push("enums");
        }
        if self.metadata.has_structs {
            submodules.push("structs");
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

            let content = quote! {
                #mod_defs

                #mod_uses
            };

            prettyplease::unparse(&syn::parse2(content)?)
        } else {
            String::new()
        })
    }

    fn emit_cmds_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("cmds.rs"))?);
        writer.write(self.cmds_module_content()?.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    fn cmds_module_content(&self) -> Result<String, EmitError> {
        Ok(String::new())
    }

    fn emit_enums_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("enums.rs"))?);
        writer.write(self.enums_module_content()?.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    fn enums_module_content(&self) -> Result<String, EmitError> {
        Ok(String::new())
    }

    fn emit_structs_module(&self, settings: &EmitSettings) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(settings.output_dir.join("structs.rs"))?);
        writer.write(self.structs_module_content()?.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    fn structs_module_content(&self) -> Result<String, EmitError> {
        for (type_info, struct_type) in self.types.items.iter().filter_map(|(_, ty)| {
            if let TypeKind::Struct(struct_type) = &ty.kind {
                Some((&ty.info, struct_type))
            } else {
                None
            }
        }) {
            let _ = self.struct_defn(type_info, struct_type)?;
        }

        Ok(String::new())
    }

    fn struct_defn(
        &self,
        type_info: &TypeInfo,
        struct_type: &StructType,
    ) -> Result<String, EmitError> {
        let struct_name = Ident::new(&type_info.output_name, Span::call_site());

        for member in struct_type.members.iter() {
            let _ = self.struct_member_defn(type_info, struct_type, member)?;
        }

        let _ = quote! {
            pub struct #struct_name {

            }
        };

        Ok(String::new())
    }

    fn struct_member_defn(
        &self,
        type_info: &TypeInfo,
        struct_type: &StructType,
        member: &StructMember,
    ) -> Result<String, EmitError> {
        let visibility = Visibility::Public(Default::default());
        let member_name = Ident::new(&member.output_name, Span::call_site());
        let member_type = Ident::new("u32", Span::call_site());

        _ = type_info;
        _ = struct_type;

        let _ = quote! {
            #visibility #member_name: #member_type,
        };

        Ok(String::new())
    }
}
