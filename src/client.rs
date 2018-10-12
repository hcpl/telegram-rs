use futures::Future;
use mtproto::{
    network::connection::Connection,
    network::sender::{SenderBuilder, SenderConnected, SenderDisconnected},
    schema::{functions, LAYER},
};
use os_type;
use serde::ser::Serialize;

use ::app_info::AppInfo;
use ::error::{self, ErrorKind};


const DEFAULT_REQUEST_RETRIES: usize = 5;
const DEFAULT_CONNECTION_RETRIES: usize = 5;

// Let's assume that the program using this lib won't ever be switching OS while running.
lazy_static! {
    static ref OS_INFO: os_type::OSInformation = os_type::current_platform();

    static ref DEFAULT_DEVICE_MODEL: &'static str = match OS_INFO.os_type {
        os_type::OSType::Unknown  => "Unknown",
        os_type::OSType::Redhat   => "Redhat",
        os_type::OSType::OSX      => "OSX",
        os_type::OSType::Ubuntu   => "Ubuntu",
        os_type::OSType::Debian   => "Debian",
        os_type::OSType::Arch     => "Arch",
        os_type::OSType::Manjaro  => "Manjaro",
        os_type::OSType::CentOS   => "CentOS",
        os_type::OSType::OpenSUSE => "OpenSUSE",
    };
    static ref DEFAULT_SYSTEM_VERSION: &'static str = &OS_INFO.version;
}

const DEFAULT_APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LANG_CODE: &str = "en";
const DEFAULT_SYSTEM_LANG_CODE: &str = "en";


#[derive(Debug, Default)]
pub struct ClientBuilder {
    app_info: Option<AppInfo>,
    request_retries: Option<usize>,
    connection_retries: Option<usize>,
    device_model: Option<String>,
    system_version: Option<String>,
    app_version: Option<String>,
    lang_code: Option<String>,
    system_lang_code: Option<String>,
    sender_builder: SenderBuilder,
}

impl ClientBuilder {
    pub fn app_info(&mut self, app_info: AppInfo) -> &mut Self {
        self.app_info = Some(app_info);
        self
    }

    pub fn request_retries(&mut self, request_retries: usize) -> &mut Self {
        self.request_retries = Some(request_retries);
        self
    }

    pub fn connection_retries(&mut self, connection_retries: usize) -> &mut Self {
        self.connection_retries = Some(connection_retries);
        self
    }

    pub fn device_model(&mut self, device_model: String) -> &mut Self {
        self.device_model = Some(device_model);
        self
    }

    pub fn system_version(&mut self, system_version: String) -> &mut Self {
        self.system_version = Some(system_version);
        self
    }

    pub fn app_version(&mut self, app_version: String) -> &mut Self {
        self.app_version = Some(app_version);
        self
    }

    pub fn lang_code(&mut self, lang_code: String) -> &mut Self {
        self.lang_code = Some(lang_code);
        self
    }

    pub fn system_lang_code(&mut self, system_lang_code: String) -> &mut Self {
        self.system_lang_code = Some(system_lang_code);
        self
    }

    pub fn sender_builder(&mut self) -> &mut SenderBuilder {
        &mut self.sender_builder
    }

    pub fn build(&self) -> error::Result<ClientDisconnected> {
        Ok(ClientDisconnected {
            app_info: self.app_info.clone()
                .ok_or_else(|| ErrorKind::ClientNoAppInfo)?,
            request_retries: self.request_retries
                .unwrap_or(DEFAULT_REQUEST_RETRIES),
            connection_retries: self.connection_retries
                .unwrap_or(DEFAULT_CONNECTION_RETRIES),
            device_model: self.device_model.as_ref().map(String::as_str)
                .unwrap_or(*DEFAULT_DEVICE_MODEL).to_owned(),
            system_version: self.system_version.as_ref().map(String::as_str)
                .unwrap_or(*DEFAULT_SYSTEM_VERSION).to_owned(),
            app_version: self.app_version.as_ref().map(String::as_str)
                .unwrap_or(DEFAULT_APP_VERSION).to_owned(),
            lang_code: self.lang_code.as_ref().map(String::as_str)
                .unwrap_or(DEFAULT_LANG_CODE).to_owned(),
            system_lang_code: self.system_lang_code.as_ref().map(String::as_str)
                .unwrap_or(DEFAULT_SYSTEM_LANG_CODE).to_owned(),
            sender_disconnd: self.sender_builder.build(),
        })
    }
}


#[derive(Debug)]
pub struct ClientDisconnected {
    app_info: AppInfo,
    request_retries: usize,
    connection_retries: usize,
    device_model: String,
    system_version: String,
    app_version: String,
    lang_code: String,
    system_lang_code: String,
    sender_disconnd: SenderDisconnected,
}

#[derive(Debug)]
pub struct ClientConnected {
    app_info: AppInfo,
    request_retries: usize,
    connection_retries: usize,
    device_model: String,
    system_version: String,
    app_version: String,
    lang_code: String,
    system_lang_code: String,
    sender_connd: SenderConnected,
}

impl ClientDisconnected {
    pub fn connect<C>(self) -> impl Future<Item = ClientConnected, Error = error::Error>
    where
        C: Connection,
    {
        let Self {
            app_info,
            request_retries, connection_retries,
            device_model, system_version, app_version,
            lang_code, system_lang_code,
            sender_disconnd,
        } = self;

        sender_disconnd.connect::<C>().and_then(move |sender_connd| {
            let get_config = init_with(
                &app_info,
                &device_model,
                &system_version,
                &app_version,
                &system_lang_code,
                &lang_code,
                functions::help::getConfig {},
            );

            let sender_connd = sender_connd.send(get_config)?;

            Ok(ClientConnected {
                app_info,
                request_retries, connection_retries,
                device_model, system_version, app_version,
                lang_code, system_lang_code,
                sender_connd,
            })
        }).map_err(Into::into)
    }
}

impl ClientConnected {
    pub fn disconnect(self) -> impl Future<Item = ClientDisconnected, Error = error::Error> {
        let Self {
            app_info,
            request_retries, connection_retries,
            device_model, system_version, app_version,
            lang_code, system_lang_code,
            sender_connd,
        } = self;

        sender_connd.disconnect().map(move |sender_disconnd| {
            ClientDisconnected {
                app_info,
                request_retries, connection_retries,
                device_model, system_version, app_version,
                lang_code, system_lang_code,
                sender_disconnd,
            }
        }).map_err(Into::into)
    }
}


fn init_with<T>(
    app_info: &AppInfo,
    device_model: &str,
    system_version: &str,
    app_version: &str,
    system_lang_code: &str,
    lang_code: &str,
    send_data: T,
) -> functions::invokeWithLayer<functions::initConnection<T>>
where
    T: Serialize,
{
    functions::invokeWithLayer {
        layer: LAYER,
        query: functions::initConnection {
            api_id: app_info.api_id,
            device_model: device_model.to_owned(),
            system_version: system_version.to_owned(),
            app_version: app_version.to_owned(),
            system_lang_code: system_lang_code.to_owned(),
            lang_pack: String::new(),  // "langPacks are for official apps only"
            lang_code: lang_code.to_owned(),
            query: send_data,
        },
    }
}
