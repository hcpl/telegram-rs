error_chain!{
    links {
        SerdeMtProto(::serde_mtproto::Error, ::serde_mtproto::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
        Hyper(::hyper::Error);
    }
}
