use proc_macro2::TokenStream;

pub trait Structured {
    fn structured(&self) -> TokenStream;
}

pub trait Reflected {
    fn reflected(&self) -> TokenStream;
}

pub trait Names {
    fn origin_name(&self) -> String;
    fn packet_name(&self) -> String;
}

pub trait ReferredPacket {
    fn referred(&self) -> TokenStream;
}

pub trait StaticPacket {
    fn r#static(&self) -> TokenStream;
}
