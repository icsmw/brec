use crate::*;

pub struct ProjectFile<'a> {
    model: &'a Model,
}

impl<'a> ProjectFile<'a> {
    pub const FILE_NAME: &'static str = "Protocol.csproj";

    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl FileName for ProjectFile<'_> {
    const FILE_NAME: &'static str = Self::FILE_NAME;
}

impl SourceWritable for ProjectFile<'_> {
    fn write(&self, writer: &mut SourceWriter) -> Result<(), Error> {
        writer.ln("<Project Sdk=\"Microsoft.NET.Sdk\">")?;
        writer.tab();
        writer.ln("<PropertyGroup>")?;
        writer.tab();
        writer.ln("<TargetFramework>net8.0</TargetFramework>")?;
        writer.ln("<ImplicitUsings>enable</ImplicitUsings>")?;
        writer.ln("<Nullable>enable</Nullable>")?;
        writer.ln(format!(
            "<AssemblyName>{}</AssemblyName>",
            self.model.package
        ))?;
        writer.ln(format!(
            "<RootNamespace>{}</RootNamespace>",
            namespace_name(&self.model.package)
        ))?;
        writer.back();
        writer.ln("</PropertyGroup>")?;
        writer.ln("<ItemGroup>")?;
        writer.tab();
        writer.ln("<None Include=\"native/*\" CopyToOutputDirectory=\"PreserveNewest\" Link=\"%(Filename)%(Extension)\" />")?;
        writer.back();
        writer.ln("</ItemGroup>")?;
        writer.back();
        writer.ln("</Project>")
    }
}

pub fn namespace_name(package: &str) -> String {
    let mut out = String::new();
    for part in package.split(|ch: char| !ch.is_ascii_alphanumeric()) {
        if part.is_empty() {
            continue;
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push_str(&first.to_uppercase().collect::<String>());
            out.push_str(chars.as_str());
        }
    }
    if out.is_empty() {
        "Protocol".to_owned()
    } else {
        out
    }
}
