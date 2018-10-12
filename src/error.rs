error_chain! {
    links {
        MtProto(::mtproto::Error, ::mtproto::ErrorKind);
    }

    foreign_links {
        Envy(::envy::Error);
        Io(::std::io::Error);
        TomlDeserialize(::toml::de::Error);
    }

    errors {
        ClientNoAppInfo {
            description("no app info provided for building `Client`")
            display("no app info provided for building `Client`")
        }
    }
}
