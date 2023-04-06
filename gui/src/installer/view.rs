use iced::widget::{
    checkbox, container, pick_list, scrollable, scrollable::Properties, slider, Space, TextInput,
};
use iced::{alignment, Alignment, Length};

use std::{collections::HashSet, str::FromStr};

use liana::miniscript::bitcoin;
use liana_ui::{
    color,
    component::{
        button, card, collapse, form, hw, separation,
        text::{text, Text},
        tooltip,
    },
    icon, theme,
    util::Collection,
    widget::*,
};

use crate::{
    hw::HardwareWallet,
    installer::{
        context::Context,
        message::{self, Message},
        prompt, Error,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Network {
    Mainnet,
    Testnet,
    Regtest,
    Signet,
}

impl From<bitcoin::Network> for Network {
    fn from(n: bitcoin::Network) -> Self {
        match n {
            bitcoin::Network::Bitcoin => Network::Mainnet,
            bitcoin::Network::Testnet => Network::Testnet,
            bitcoin::Network::Regtest => Network::Regtest,
            bitcoin::Network::Signet => Network::Signet,
        }
    }
}

impl From<Network> for bitcoin::Network {
    fn from(network: Network) -> bitcoin::Network {
        match network {
            Network::Mainnet => bitcoin::Network::Bitcoin,
            Network::Testnet => bitcoin::Network::Testnet,
            Network::Regtest => bitcoin::Network::Regtest,
            Network::Signet => bitcoin::Network::Signet,
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "Bitcoin mainnet"),
            Self::Testnet => write!(f, "Bitcoin testnet"),
            Self::Regtest => write!(f, "Bitcoin regtest"),
            Self::Signet => write!(f, "Bitcoin signet"),
        }
    }
}

const NETWORKS: [Network; 4] = [
    Network::Mainnet,
    Network::Testnet,
    Network::Signet,
    Network::Regtest,
];

pub fn welcome<'a>() -> Element<'a, Message> {
    Container::new(Container::new(
        Column::new()
            .push(
                Row::new()
                    .spacing(20)
                    .push(
                        Button::new(
                            Container::new(
                                Column::new()
                                    .width(Length::Units(250))
                                    .push(icon::wallet_icon().size(50).width(Length::Units(100)))
                                    .push(text("Create a new wallet"))
                                    .align_items(Alignment::Center),
                            )
                            .padding(20),
                        )
                        .style(theme::Button::Secondary)
                        .on_press(Message::CreateWallet),
                    )
                    .push(
                        Button::new(
                            Container::new(
                                Column::new()
                                    .width(Length::Units(250))
                                    .push(icon::people_icon().size(50).width(Length::Units(100)))
                                    .push(text("Participate in a new wallet"))
                                    .align_items(Alignment::Center),
                            )
                            .padding(20),
                        )
                        .style(theme::Button::Secondary)
                        .on_press(Message::ParticipateWallet),
                    )
                    .push(
                        Button::new(
                            Container::new(
                                Column::new()
                                    .width(Length::Units(250))
                                    .push(icon::import_icon().size(50).width(Length::Units(100)))
                                    .push(text("Import a wallet backup"))
                                    .align_items(Alignment::Center),
                            )
                            .padding(20),
                        )
                        .style(theme::Button::Secondary)
                        .on_press(Message::ImportWallet),
                    ),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(50)
            .align_items(Alignment::Center),
    ))
    .center_y()
    .center_x()
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

#[allow(clippy::too_many_arguments)]
pub fn define_descriptor<'a>(
    progress: (usize, usize),
    network: bitcoin::Network,
    network_valid: bool,
    spending_keys: Vec<Element<'a, Message>>,
    spending_threshold: usize,
    recovery_paths: Vec<Element<'a, Message>>,
    valid: bool,
    error: Option<&String>,
) -> Element<'a, Message> {
    let row_network = Row::new()
        .spacing(10)
        .align_items(Alignment::Center)
        .push(text("Network:").bold())
        .push(container(
            pick_list(&NETWORKS[..], Some(Network::from(network)), |net| {
                Message::Network(net.into())
            })
            .style(if network_valid {
                theme::PickList::Simple
            } else {
                theme::PickList::Invalid
            })
            .padding(10),
        ))
        .push_maybe(if network_valid {
            None
        } else {
            Some(
                text("A data directory already exists for this network")
                    .style(color::legacy::ALERT),
            )
        })
        .padding(50);

    let col_spending_keys = Column::new()
        .push(
            Row::new()
                .spacing(10)
                .push(Space::with_width(Length::Units(5)))
                .push(text("Primary path:").bold())
                .push(tooltip(prompt::DEFINE_DESCRIPTOR_PRIMARY_PATH_TOOLTIP)),
        )
        .push(Container::new(
            Row::new()
                .align_items(Alignment::Center)
                .push_maybe(if spending_keys.len() > 1 {
                    Some(threshsold_input::threshsold_input(
                        spending_threshold,
                        spending_keys.len(),
                        |value| {
                            Message::DefineDescriptor(message::DefineDescriptor::PrimaryPath(
                                message::DefinePath::ThresholdEdited(value),
                            ))
                        },
                    ))
                } else {
                    None
                })
                .push(
                    scrollable(
                        Row::new()
                            .spacing(5)
                            .align_items(Alignment::Center)
                            .push(Row::with_children(spending_keys).spacing(5))
                            .push(
                                Button::new(
                                    Container::new(icon::plus_icon().size(50))
                                        .width(Length::Units(150))
                                        .height(Length::Units(150))
                                        .align_y(alignment::Vertical::Center)
                                        .align_x(alignment::Horizontal::Center),
                                )
                                .width(Length::Units(150))
                                .height(Length::Units(150))
                                .style(theme::Button::TransparentBorder)
                                .on_press(
                                    Message::DefineDescriptor(
                                        message::DefineDescriptor::PrimaryPath(
                                            message::DefinePath::AddKey,
                                        ),
                                    ),
                                ),
                            )
                            .padding(5),
                    )
                    .horizontal_scroll(Properties::new().width(3).scroller_width(3)),
                ),
        ))
        .spacing(10);

    layout(
        progress,
        Column::new()
            .push(Space::with_height(Length::Units(30)))
            .push(text("Create the wallet").bold().size(50))
            .push(
                Column::new()
                    .width(Length::Fill)
                    .align_items(Alignment::Center)
                    .push(row_network)
                    .push(
                        Column::new()
                            .spacing(25)
                            .push(col_spending_keys)
                            .push(
                                Row::new()
                                    .spacing(10)
                                    .push(Space::with_width(Length::Units(5)))
                                    .push(text("Recovery paths:").bold())
                                    .push(tooltip(prompt::DEFINE_DESCRIPTOR_RECOVERY_PATH_TOOLTIP)),
                            )
                            .push(Column::with_children(recovery_paths).spacing(10)),
                    )
                    .spacing(25),
            )
            .push(
                Row::new()
                    .spacing(10)
                    .push(button::border(None, "Add a recovery path").on_press(
                        Message::DefineDescriptor(message::DefineDescriptor::AddRecoveryPath),
                    ))
                    .push(if !valid {
                        button::primary(None, "Next").width(Length::Units(200))
                    } else {
                        button::primary(None, "Next")
                            .width(Length::Units(200))
                            .on_press(Message::Next)
                    }),
            )
            .push_maybe(error.map(|e| card::error("Failed to create descriptor", e.to_string())))
            .push(Space::with_height(Length::Units(20)))
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(50)
            .align_items(Alignment::Center),
    )
}

pub fn recovery_path_view(
    sequence: u16,
    duplicate_sequence: bool,
    recovery_threshold: usize,
    recovery_keys: Vec<Element<message::DefinePath>>,
) -> Element<message::DefinePath> {
    Container::new(
        Column::new()
            .push(defined_sequence(sequence, duplicate_sequence))
            .push(
                Row::new()
                    .align_items(Alignment::Center)
                    .push_maybe(if recovery_keys.len() > 1 {
                        Some(threshsold_input::threshsold_input(
                            recovery_threshold,
                            recovery_keys.len(),
                            message::DefinePath::ThresholdEdited,
                        ))
                    } else {
                        None
                    })
                    .push(
                        scrollable(
                            Row::new()
                                .spacing(5)
                                .align_items(Alignment::Center)
                                .push(Row::with_children(recovery_keys).spacing(5))
                                .push(
                                    Button::new(
                                        Container::new(icon::plus_icon().size(50))
                                            .width(Length::Units(150))
                                            .height(Length::Units(150))
                                            .align_y(alignment::Vertical::Center)
                                            .align_x(alignment::Horizontal::Center),
                                    )
                                    .width(Length::Units(150))
                                    .height(Length::Units(150))
                                    .style(theme::Button::TransparentBorder)
                                    .on_press(message::DefinePath::AddKey),
                                )
                                .padding(5),
                        )
                        .horizontal_scroll(Properties::new().width(3).scroller_width(3)),
                    ),
            ),
    )
    .padding(5)
    .style(theme::Container::Card(theme::Card::Border))
    .into()
}

pub fn import_descriptor<'a>(
    progress: (usize, usize),
    change_network: bool,
    network: bitcoin::Network,
    network_valid: bool,
    imported_descriptor: &form::Value<String>,
    error: Option<&String>,
) -> Element<'a, Message> {
    let row_network = Row::new()
        .spacing(10)
        .align_items(Alignment::Center)
        .push(text("Network:").bold())
        .push(Container::new(
            pick_list(&NETWORKS[..], Some(Network::from(network)), |net| {
                Message::Network(net.into())
            })
            .style(if network_valid {
                theme::PickList::Simple
            } else {
                theme::PickList::Invalid
            })
            .padding(10),
        ))
        .push_maybe(if network_valid {
            None
        } else {
            Some(
                text("A data directory already exists for this network")
                    .style(color::legacy::ALERT),
            )
        });
    let col_descriptor = Column::new()
        .push(text("Descriptor:").bold())
        .push(
            form::Form::new("Descriptor", imported_descriptor, |msg| {
                Message::DefineDescriptor(message::DefineDescriptor::ImportDescriptor(msg))
            })
            .warning("Incompatible descriptor. Note that starting from v0.2 Liana requires extended keys in a descriptor to have an origin.")
            .size(20)
            .padding(10),
        )
        .spacing(10);
    layout(
        progress,
        Column::new()
            .push(text("Import the wallet").bold().size(50))
            .push(
                Column::new()
                    .spacing(20)
                    .push_maybe(if change_network {
                        Some(row_network)
                    } else {
                        None
                    })
                    .push(col_descriptor),
            )
            .push(if imported_descriptor.value.is_empty() {
                button::primary(None, "Next").width(Length::Units(200))
            } else {
                button::primary(None, "Next")
                    .width(Length::Units(200))
                    .on_press(Message::Next)
            })
            .push_maybe(error.map(|e| card::error("Invalid descriptor", e.to_string())))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Alignment::Center),
    )
}

pub fn signer_xpubs(xpubs: &Vec<String>) -> Element<Message> {
    Container::new(
        Column::new()
            .push(
                Button::new(
                    Row::new().align_items(Alignment::Center).push(
                        Column::new()
                            .push(text("This computer").bold())
                            .push(
                                text("Derive a key from a mnemonic stored on this computer")
                                    .small(),
                            )
                            .spacing(5)
                            .width(Length::Fill),
                    ),
                )
                .on_press(Message::UseHotSigner)
                .padding(10)
                .style(theme::Button::TransparentBorder)
                .width(Length::Fill),
            )
            .push_maybe(if xpubs.is_empty() {
                None
            } else {
                Some(separation().width(Length::Fill))
            })
            .push_maybe(if xpubs.is_empty() {
                None
            } else {
                Some(xpubs.iter().fold(Column::new().padding(15), |col, xpub| {
                    col.push(
                        Row::new()
                            .spacing(5)
                            .align_items(Alignment::Center)
                            .push(
                                Container::new(
                                    scrollable(Container::new(text(xpub).small()).padding(10))
                                        .horizontal_scroll(
                                            Properties::new().width(2).scroller_width(2),
                                        ),
                                )
                                .width(Length::Fill),
                            )
                            .push(
                                Container::new(
                                    button::border(Some(icon::clipboard_icon()), "Copy")
                                        .on_press(Message::Clibpboard(xpub.clone()))
                                        .width(Length::Shrink),
                                )
                                .padding(10),
                            ),
                    )
                }))
            })
            .push_maybe(if !xpubs.is_empty() {
                Some(
                    Container::new(
                        button::border(Some(icon::plus_icon()), "New public key")
                            .on_press(Message::UseHotSigner),
                    )
                    .padding(10),
                )
            } else {
                None
            }),
    )
    .style(theme::Container::Card(theme::Card::Simple))
    .into()
}

pub fn hardware_wallet_xpubs<'a>(
    i: usize,
    xpubs: &'a Vec<String>,
    hw: &'a HardwareWallet,
    processing: bool,
    error: Option<&Error>,
) -> Element<'a, Message> {
    let mut bttn = Button::new(match hw {
        HardwareWallet::Supported {
            kind,
            version,
            fingerprint,
            alias,
            ..
        } => {
            if processing {
                hw::processing_hardware_wallet(kind, version.as_ref(), fingerprint, alias.as_ref())
            } else {
                hw::supported_hardware_wallet(kind, version.as_ref(), fingerprint, alias.as_ref())
            }
        }
        HardwareWallet::Unsupported { version, kind, .. } => {
            hw::unsupported_hardware_wallet(&kind.to_string(), version.as_ref())
        }
    })
    .style(theme::Button::Secondary)
    .width(Length::Fill);
    if !processing && hw.is_supported() {
        bttn = bttn.on_press(Message::Select(i));
    }
    Container::new(
        Column::new()
            .push_maybe(error.map(|e| card::warning(e.to_string()).width(Length::Fill)))
            .push(bttn)
            .push_maybe(if xpubs.is_empty() {
                None
            } else {
                Some(separation().width(Length::Fill))
            })
            .push_maybe(if xpubs.is_empty() {
                None
            } else {
                Some(xpubs.iter().fold(Column::new().padding(15), |col, xpub| {
                    col.push(
                        Row::new()
                            .spacing(5)
                            .align_items(Alignment::Center)
                            .push(
                                Container::new(
                                    scrollable(Container::new(text(xpub).small()).padding(10))
                                        .horizontal_scroll(
                                            Properties::new().width(2).scroller_width(2),
                                        ),
                                )
                                .width(Length::Fill),
                            )
                            .push(
                                Container::new(
                                    button::border(Some(icon::clipboard_icon()), "Copy")
                                        .on_press(Message::Clibpboard(xpub.clone()))
                                        .width(Length::Shrink),
                                )
                                .padding(10),
                            ),
                    )
                }))
            })
            .push_maybe(if !xpubs.is_empty() {
                Some(
                    Container::new(if !processing {
                        button::border(Some(icon::plus_icon()), "New public key")
                            .on_press(Message::Select(i))
                    } else {
                        button::border(Some(icon::plus_icon()), "New public key")
                    })
                    .padding(10),
                )
            } else {
                None
            }),
    )
    .style(theme::Container::Card(theme::Card::Simple))
    .into()
}

pub fn participate_xpub<'a>(
    progress: (usize, usize),
    network: bitcoin::Network,
    network_valid: bool,
    hws: Vec<Element<'a, Message>>,
    signer: Element<'a, Message>,
    shared: bool,
) -> Element<'a, Message> {
    let row_network = Row::new()
        .spacing(10)
        .align_items(Alignment::Center)
        .push(text("Network:").bold())
        .push(Container::new(
            pick_list(&NETWORKS[..], Some(Network::from(network)), |net| {
                Message::Network(net.into())
            })
            .style(if network_valid {
                theme::PickList::Simple
            } else {
                theme::PickList::Invalid
            })
            .padding(10),
        ))
        .push_maybe(if network_valid {
            None
        } else {
            Some(
                text("A data directory already exists for this network")
                    .style(color::legacy::ALERT),
            )
        });

    layout(
        progress,
        Column::new()
            .push(text("Share your public keys").bold().size(50))
            .push(
                Column::new()
                    .spacing(20)
                    .width(Length::Fill)
                    .push(row_network),
            )
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .spacing(10)
                            .align_items(Alignment::Center)
                            .push(
                                Container::new(text("Generate an extended public key by selecting a signing device:").bold())
                                    .width(Length::Fill),
                            )
                            .push(
                                button::border(Some(icon::reload_icon()), "Refresh")
                                    .on_press(Message::Reload),
                            ),
                    )
                    .spacing(10)
                    .push(Column::with_children(hws).spacing(10))
                    .push(signer)
                    .width(Length::Fill),
            )
            .push(checkbox(
                "I have shared my public keys",
                shared,
                Message::UserActionDone,
            ))
            .push(if shared {
                button::primary(None, "Next")
                    .width(Length::Units(200))
                    .on_press(Message::Next)
            } else {
                button::primary(None, "Next").width(Length::Units(200))
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Alignment::Center),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn register_descriptor<'a>(
    progress: (usize, usize),
    descriptor: String,
    hws: &'a [HardwareWallet],
    registered: &HashSet<bitcoin::util::bip32::Fingerprint>,
    error: Option<&Error>,
    processing: bool,
    chosen_hw: Option<usize>,
    done: bool,
) -> Element<'a, Message> {
    layout(
        progress,
        Column::new()
            .max_width(1000)
            .push(text("Register descriptor").bold().size(50))
            .push(card::simple(
                Column::new()
                    .push(text("The descriptor:").small().bold())
                    .push(text(descriptor.clone()).small())
                    .push(
                        Row::new().push(Column::new().width(Length::Fill)).push(
                            button::transparent_border(Some(icon::clipboard_icon()), "Copy")
                                .on_press(Message::Clibpboard(descriptor)),
                        ),
                    )
                    .spacing(10),
            ))
            .push(text(prompt::REGISTER_DESCRIPTOR_HELP))
            .push_maybe(error.map(|e| card::error("Failed to register descriptor", e.to_string())))
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .spacing(10)
                            .align_items(Alignment::Center)
                            .push(
                                Container::new(
                                    text("Select hardware wallet to register descriptor on:")
                                        .bold(),
                                )
                                .width(Length::Fill),
                            )
                            .push(
                                button::border(Some(icon::reload_icon()), "Refresh")
                                    .on_press(Message::Reload),
                            ),
                    )
                    .spacing(10)
                    .push(
                        hws.iter()
                            .enumerate()
                            .fold(Column::new().spacing(10), |col, (i, hw)| {
                                col.push(hw_list_view(
                                    i,
                                    hw,
                                    Some(i) == chosen_hw,
                                    processing,
                                    hw.fingerprint()
                                        .map(|fg| registered.contains(&fg))
                                        .unwrap_or(false),
                                ))
                            }),
                    )
                    .width(Length::Fill),
            )
            .push(checkbox(
                "I have registered the descriptor on my device(s)",
                done,
                Message::UserActionDone,
            ))
            .push(if done && !processing {
                button::primary(None, "Next")
                    .on_press(Message::Next)
                    .width(Length::Units(200))
            } else {
                button::primary(None, "Next").width(Length::Units(200))
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Alignment::Center),
    )
}

pub fn backup_descriptor<'a>(
    progress: (usize, usize),
    descriptor: String,
    done: bool,
) -> Element<'a, Message> {
    layout(
        progress,
        Column::new()
            .push(
                text("Did you backup your wallet descriptor ?")
                    .bold()
                    .size(50),
            )
            .push(
                Column::new()
                    .push(text(prompt::BACKUP_DESCRIPTOR_MESSAGE))
                    .push(collapse::Collapse::new(
                        || {
                            Button::new(
                                Row::new()
                                    .align_items(Alignment::Center)
                                    .spacing(10)
                                    .push(text("Learn more").small().bold())
                                    .push(icon::collapse_icon()),
                            )
                            .style(theme::Button::Transparent)
                        },
                        || {
                            Button::new(
                                Row::new()
                                    .align_items(Alignment::Center)
                                    .spacing(10)
                                    .push(text("Learn more").small().bold())
                                    .push(icon::collapsed_icon()),
                            )
                            .style(theme::Button::Transparent)
                        },
                        help_backup,
                    ))
                    .max_width(1000),
            )
            .push(card::simple(
                Column::new()
                    .push(text("The descriptor:").small().bold())
                    .push(text(descriptor.clone()).small())
                    .push(
                        Row::new().push(Column::new().width(Length::Fill)).push(
                            button::transparent_border(Some(icon::clipboard_icon()), "Copy")
                                .on_press(Message::Clibpboard(descriptor)),
                        ),
                    )
                    .spacing(10)
                    .max_width(1000),
            ))
            .push(checkbox(
                "I have backed up my descriptor",
                done,
                Message::UserActionDone,
            ))
            .push(if done {
                button::primary(None, "Next")
                    .on_press(Message::Next)
                    .width(Length::Units(200))
            } else {
                button::primary(None, "Next").width(Length::Units(200))
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Alignment::Center),
    )
}

pub fn help_backup<'a>() -> Element<'a, Message> {
    text(prompt::BACKUP_DESCRIPTOR_HELP).small().into()
}

pub fn define_bitcoin<'a>(
    progress: (usize, usize),
    address: &form::Value<String>,
    cookie_path: &form::Value<String>,
    is_running: Option<&Result<(), Error>>,
) -> Element<'a, Message> {
    let col_address = Column::new()
        .push(text("Address:").bold())
        .push(
            form::Form::new("Address", address, |msg| {
                Message::DefineBitcoind(message::DefineBitcoind::AddressEdited(msg))
            })
            .warning("Please enter correct address")
            .size(20)
            .padding(10),
        )
        .spacing(10);

    let col_cookie = Column::new()
        .push(text("Cookie path:").bold())
        .push(
            form::Form::new("Cookie path", cookie_path, |msg| {
                Message::DefineBitcoind(message::DefineBitcoind::CookiePathEdited(msg))
            })
            .warning("Please enter correct path")
            .size(20)
            .padding(10),
        )
        .spacing(10);

    layout(
        progress,
        Column::new()
            .push(
                text("Set up connection to the Bitcoin full node")
                    .bold()
                    .size(50),
            )
            .push(col_address)
            .push(col_cookie)
            .push_maybe(if is_running.is_some() {
                is_running.map(|res| {
                    if res.is_ok() {
                        Container::new(
                            Row::new()
                                .spacing(10)
                                .align_items(Alignment::Center)
                                .push(icon::circle_check_icon().style(color::legacy::SUCCESS))
                                .push(text("Connection checked").style(color::legacy::SUCCESS)),
                        )
                    } else {
                        Container::new(
                            Row::new()
                                .spacing(10)
                                .align_items(Alignment::Center)
                                .push(icon::circle_cross_icon().style(color::legacy::ALERT))
                                .push(text("Connection failed").style(color::legacy::ALERT)),
                        )
                    }
                })
            } else {
                Some(Container::new(Space::with_height(Length::Units(25))))
            })
            .push(
                Row::new()
                    .spacing(10)
                    .push(Container::new(
                        button::border(None, "Check connection")
                            .on_press(Message::DefineBitcoind(
                                message::DefineBitcoind::PingBitcoind,
                            ))
                            .width(Length::Units(200)),
                    ))
                    .push(if is_running.map(|res| res.is_ok()).unwrap_or(false) {
                        button::primary(None, "Next")
                            .on_press(Message::Next)
                            .width(Length::Units(200))
                    } else {
                        button::primary(None, "Next").width(Length::Units(200))
                    }),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Alignment::Center),
    )
}

pub fn install<'a>(
    progress: (usize, usize),
    context: &Context,
    descriptor: String,
    generating: bool,
    config_path: Option<&std::path::PathBuf>,
    warning: Option<&'a String>,
) -> Element<'a, Message> {
    layout(
        progress,
        Column::new()
            .push(Space::with_height(Length::Units(50)))
            .push(text("Final step").bold().size(50))
            .push(Space::with_height(Length::Units(50)))
            .push(text(
                "Check your information before finalizing the install process:",
            ))
            .push(
                Container::new(
                    Column::new()
                        .spacing(10)
                        .push(
                            card::simple(
                                Column::new()
                                    .spacing(5)
                                    .push(text("Descriptor:").small().bold())
                                    .push(text(descriptor).small()),
                            )
                            .width(Length::Fill),
                        )
                        .push_maybe(if context.hws.is_empty() && context.signer.is_none() {
                            None
                        } else {
                            Some(
                                card::simple(
                                    Column::new()
                                        .spacing(5)
                                        .push(text("Registered signing devices:").small().bold())
                                        .push_maybe(if context.hws.is_empty() {
                                            None
                                        } else {
                                            Some(context.hws.iter().fold(
                                                Column::new(),
                                                |acc, hw| {
                                                    acc.push(
                                                        Row::new()
                                                            .spacing(5)
                                                            .push(text(hw.0.to_string()).small())
                                                            .push(
                                                                text(format!(
                                                                    "(fingerprint: {})",
                                                                    hw.1
                                                                ))
                                                                .small(),
                                                            ),
                                                    )
                                                },
                                            ))
                                        })
                                        .push_maybe(context.signer.as_ref().map(|signer| {
                                            Row::new().push(text("This computer").small()).push(
                                                text(format!(
                                                    "(fingerprint: {})",
                                                    signer.fingerprint()
                                                ))
                                                .small(),
                                            )
                                        })),
                                )
                                .width(Length::Fill),
                            )
                        })
                        .push(
                            card::simple(
                                Column::new()
                                    .push(text("Bitcoind:").small().bold())
                                    .push(
                                        Row::new()
                                            .spacing(5)
                                            .align_items(Alignment::Center)
                                            .push(text("Cookie path:").small())
                                            .push(
                                                text(format!(
                                                    "{}",
                                                    context
                                                        .bitcoind_config
                                                        .as_ref()
                                                        .unwrap()
                                                        .cookie_path
                                                        .to_string_lossy()
                                                ))
                                                .small(),
                                            ),
                                    )
                                    .push(
                                        Row::new()
                                            .spacing(5)
                                            .align_items(Alignment::Center)
                                            .push(text("Address:").small())
                                            .push(
                                                text(format!(
                                                    "{}",
                                                    context.bitcoind_config.as_ref().unwrap().addr
                                                ))
                                                .small(),
                                            ),
                                    ),
                            )
                            .width(Length::Fill),
                        ),
                )
                .max_width(1000),
            )
            .push(Space::with_height(Length::Units(50)))
            .push_maybe(warning.map(|e| card::invalid(text(e))))
            .push(if generating {
                Container::new(button::primary(None, "Installing ...").width(Length::Units(200)))
            } else if let Some(path) = config_path {
                Container::new(
                    Column::new()
                        .push(Container::new(text("Installed !")))
                        .push(Container::new(
                            button::primary(None, "Start")
                                .on_press(Message::Exit(path.clone()))
                                .width(Length::Units(200)),
                        ))
                        .align_items(Alignment::Center)
                        .spacing(20),
                )
                .padding(50)
                .width(Length::Fill)
                .center_x()
            } else {
                Container::new(
                    button::primary(None, "Finalize installation")
                        .on_press(Message::Install)
                        .width(Length::Units(200)),
                )
            })
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center),
    )
}

pub fn defined_sequence<'a>(
    sequence: u16,
    duplicate_sequence: bool,
) -> Element<'a, message::DefinePath> {
    let (n_years, n_months, n_days, n_hours, n_minutes) = duration_from_sequence(sequence);
    Container::new(
        Column::new()
            .spacing(5)
            .push_maybe(if duplicate_sequence {
                Some(
                    text("No two recovery paths may become available at the very same date.")
                        .small()
                        .style(color::legacy::ALERT),
                )
            } else {
                None
            })
            .push(
                Row::new()
                    .align_items(Alignment::Center)
                    .push(
                        Container::new(
                            Column::new()
                                .spacing(5)
                                .push(text(format!("Available after {} blocks", sequence)).bold())
                                .push(
                                    [
                                        (n_years, "y"),
                                        (n_months, "m"),
                                        (n_days, "d"),
                                        (n_hours, "h"),
                                        (n_minutes, "mn"),
                                    ]
                                    .iter()
                                    .fold(
                                        Row::new().spacing(5),
                                        |row, (n, unit)| {
                                            row.push_maybe(if *n > 0 {
                                                Some(text(format!("{}{}", n, unit,)))
                                            } else {
                                                None
                                            })
                                        },
                                    ),
                                ),
                        )
                        .padding(5)
                        .align_y(alignment::Vertical::Center),
                    )
                    .push(
                        button::border(Some(icon::pencil_icon()), "Edit")
                            .on_press(message::DefinePath::EditSequence),
                    )
                    .spacing(15),
            ),
    )
    .padding(5)
    .into()
}

pub fn undefined_descriptor_key<'a>() -> Element<'a, message::DefineKey> {
    card::simple(
        Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .push(
                Row::new()
                    .align_items(Alignment::Center)
                    .push(Space::with_width(Length::Fill))
                    .push(
                        Button::new(icon::cross_icon())
                            .style(theme::Button::Transparent)
                            .on_press(message::DefineKey::Delete),
                    ),
            )
            .push(
                Container::new(
                    Column::new()
                        .spacing(15)
                        .align_items(Alignment::Center)
                        .push(
                            icon::key_icon()
                                .style(color::DARK_GREY)
                                .size(30)
                                .width(Length::Units(50)),
                        ),
                )
                .height(Length::Fill)
                .align_y(alignment::Vertical::Center),
            )
            .push(
                button::border(Some(icon::pencil_icon()), "Set").on_press(message::DefineKey::Edit),
            )
            .push(Space::with_height(Length::Units(5))),
    )
    .padding(5)
    .height(Length::Units(150))
    .width(Length::Units(150))
    .into()
}

pub fn defined_descriptor_key(
    name: &str,
    valid: bool,
    duplicate_key: bool,
    duplicate_name: bool,
) -> Element<message::DefineKey> {
    let col = Column::new()
        .width(Length::Fill)
        .align_items(Alignment::Center)
        .push(
            Row::new()
                .align_items(Alignment::Center)
                .push(Space::with_width(Length::Fill))
                .push(
                    Button::new(icon::cross_icon())
                        .style(theme::Button::Transparent)
                        .on_press(message::DefineKey::Delete),
                ),
        )
        .push(
            Column::new()
                .align_items(Alignment::Center)
                .spacing(5)
                .push(
                    Container::new(
                        Column::new()
                            .spacing(5)
                            .align_items(Alignment::Center)
                            .push(
                                scrollable(text(name).bold()).horizontal_scroll(
                                    Properties::new().width(2).scroller_width(2),
                                ),
                            )
                            .push(
                                icon::circle_check_icon()
                                    .style(color::legacy::SUCCESS)
                                    .size(20)
                                    .width(Length::Units(50)),
                            ),
                    )
                    .height(Length::Fill)
                    .align_y(alignment::Vertical::Center),
                )
                .height(Length::Fill),
        )
        .push(button::border(Some(icon::pencil_icon()), "Edit").on_press(message::DefineKey::Edit))
        .push(Space::with_height(Length::Units(5)));

    if !valid {
        Column::new()
            .align_items(Alignment::Center)
            .push(
                card::invalid(col)
                    .padding(5)
                    .height(Length::Units(150))
                    .width(Length::Units(150)),
            )
            .push(
                text("Key is for a different network")
                    .small()
                    .style(color::legacy::ALERT),
            )
            .into()
    } else if duplicate_key {
        Column::new()
            .align_items(Alignment::Center)
            .push(
                card::invalid(col)
                    .padding(5)
                    .height(Length::Units(150))
                    .width(Length::Units(150)),
            )
            .push(text("Duplicate key").small().style(color::legacy::ALERT))
            .into()
    } else if duplicate_name {
        Column::new()
            .align_items(Alignment::Center)
            .push(
                card::invalid(col)
                    .padding(5)
                    .height(Length::Units(150))
                    .width(Length::Units(150)),
            )
            .push(text("Duplicate name").small().style(color::legacy::ALERT))
            .into()
    } else {
        card::simple(col)
            .padding(5)
            .height(Length::Units(150))
            .width(Length::Units(150))
            .into()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn edit_key_modal<'a>(
    network: bitcoin::Network,
    hws: &'a [HardwareWallet],
    error: Option<&Error>,
    processing: bool,
    chosen_hw: Option<usize>,
    chosen_signer: bool,
    form_xpub: &form::Value<String>,
    form_name: &'a form::Value<String>,
    edit_name: bool,
) -> Element<'a, Message> {
    Column::new()
        .push_maybe(error.map(|e| card::error("Failed to import xpub", e.to_string())))
        .push(card::simple(
            Column::new()
                .spacing(25)
                .push(
                    Column::new()
                        .push(
                            Row::new()
                                .spacing(10)
                                .align_items(Alignment::Center)
                                .push(
                                    Container::new(text("Select a signing device:").bold())
                                        .width(Length::Fill),
                                )
                                .push(
                                    button::border(Some(icon::reload_icon()), "Refresh")
                                        .on_press(Message::Reload),
                                ),
                        )
                        .spacing(10)
                        .push(hws.iter().enumerate().fold(
                            Column::new().spacing(10),
                            |col, (i, hw)| {
                                col.push(hw_list_view(
                                    i,
                                    hw,
                                    Some(i) == chosen_hw,
                                    processing,
                                    !processing
                                        && Some(i) == chosen_hw
                                        && form_xpub.valid
                                        && !form_xpub.value.is_empty(),
                                ))
                            },
                        ))
                        .push(
                            Button::new(
                                Row::new()
                                    .padding(5)
                                    .width(Length::Fill)
                                    .align_items(Alignment::Center)
                                    .push(
                                        Column::new()
                                            .spacing(5)
                                            .push(text("This computer").bold())
                                            .push(
                                                text("Derive a key from a mnemonic stored on this computer").small(),
                                            )
                                            .width(Length::Fill),
                                    )
                                    .push_maybe(if chosen_signer {
                                        Some(icon::circle_check_icon().style(color::legacy::SUCCESS))
                                    } else {
                                        None
                                    })
                                    .spacing(10),
                            )
                            .width(Length::Fill)
                            .on_press(Message::UseHotSigner)
                            .style(theme::Button::Secondary),
                        )
                        .width(Length::Fill),
                )
                .push(
                    Column::new()
                        .spacing(5)
                        .push(text("Or enter an extended public key:").bold())
                        .push(
                            Row::new()
                                .push(
                                    form::Form::new("Extended public key", form_xpub, |msg| {
                                        Message::DefineDescriptor(
                                            message::DefineDescriptor::KeyModal(message::ImportKeyModal::XPubEdited(msg)),
                                        )
                                    })
                                    .warning(if network == bitcoin::Network::Bitcoin {
                                        "Please enter correct xpub with origin"
                                    } else {
                                        "Please enter correct tpub with origin"
                                    })
                                    .size(20)
                                    .padding(10),
                                )
                                .spacing(10)
                                .push(Container::new(text("/<0;1>/*")).padding(5)),
                        ),
                )
                .push(
                    if !edit_name && !form_xpub.value.is_empty() && form_xpub.valid {
                        Column::new().push(
                            Row::new()
                                .push(
                                    Column::new()
                                        .spacing(5)
                                        .width(Length::Fill)
                                        .push(
                                            Row::new()
                                                .spacing(5)
                                                .push(text("Fingerprint alias:").bold())
                                                .push(tooltip(
                                                    prompt::DEFINE_DESCRIPTOR_FINGERPRINT_TOOLTIP,
                                                )),
                                        )
                                        .push(text(&form_name.value)),
                                )
                                .push(button::border(Some(icon::pencil_icon()), "Edit").on_press(
                                        Message::DefineDescriptor(
                                            message::DefineDescriptor::KeyModal(message::ImportKeyModal::EditName),
                                        )
                                )),
                        )
                    } else if !form_xpub.value.is_empty() && form_xpub.valid {
                        Column::new()
                            .spacing(5)
                            .push(
                                Row::new()
                                    .spacing(5)
                                    .push(text("Fingerprint alias:").bold())
                                    .push(tooltip(prompt::DEFINE_DESCRIPTOR_FINGERPRINT_TOOLTIP)),
                            )
                            .push(
                                form::Form::new("Alias", form_name, |msg| {
                                    Message::DefineDescriptor(
                                            message::DefineDescriptor::KeyModal(message::ImportKeyModal::NameEdited(msg)),
                                        )
                                })
                                .warning("Please enter correct alias")
                                .size(20)
                                .padding(10),
                            )
                    } else {
                        Column::new()
                    },
                )
                .push(
                    if form_xpub.valid && !form_xpub.value.is_empty() && !form_name.value.is_empty()
                    {
                        button::primary(None, "Apply")
                            .on_press(
                                Message::DefineDescriptor(
                                    message::DefineDescriptor::KeyModal(message::ImportKeyModal::ConfirmXpub),
                                )
                            )
                            .width(Length::Units(200))
                    } else {
                        button::primary(None, "Apply").width(Length::Units(100))
                    },
                )
                .align_items(Alignment::Center),
        ))
        .width(Length::Units(600))
        .into()
}

/// returns y,m,d,h,m
fn duration_from_sequence(sequence: u16) -> (u32, u32, u32, u32, u32) {
    let mut n_minutes = sequence as u32 * 10;
    let n_years = n_minutes / 525960;
    n_minutes -= n_years * 525960;
    let n_months = n_minutes / 43830;
    n_minutes -= n_months * 43830;
    let n_days = n_minutes / 1440;
    n_minutes -= n_days * 1440;
    let n_hours = n_minutes / 60;
    n_minutes -= n_hours * 60;

    (n_years, n_months, n_days, n_hours, n_minutes)
}

pub fn edit_sequence_modal<'a>(sequence: &form::Value<String>) -> Element<'a, Message> {
    let mut col = Column::new()
        .width(Length::Fill)
        .spacing(20)
        .align_items(Alignment::Center)
        .push(text("Activate recovery path after:"))
        .push(
            Row::new()
                .push(
                    Container::new(
                        form::Form::new("ex: 1000", sequence, |v| {
                            Message::DefineDescriptor(message::DefineDescriptor::SequenceModal(
                                message::SequenceModal::SequenceEdited(v),
                            ))
                        })
                        .warning("Sequence must be superior to 0 and inferior to 65535"),
                    )
                    .width(Length::Units(200)),
                )
                .spacing(10)
                .push(text("blocks").bold()),
        );

    if sequence.valid {
        if let Ok(sequence) = u16::from_str(&sequence.value) {
            let (n_years, n_months, n_days, n_hours, n_minutes) = duration_from_sequence(sequence);
            col = col
                .push(
                    [
                        (n_years, "year"),
                        (n_months, "month"),
                        (n_days, "day"),
                        (n_hours, "hour"),
                        (n_minutes, "minute"),
                    ]
                    .iter()
                    .fold(Row::new().spacing(5), |row, (n, unit)| {
                        row.push_maybe(if *n > 0 {
                            Some(
                                text(format!("{} {}{}", n, unit, if *n > 1 { "s" } else { "" }))
                                    .bold(),
                            )
                        } else {
                            None
                        })
                    }),
                )
                .push(
                    Container::new(slider(1..=u16::MAX, sequence, |v| {
                        Message::DefineDescriptor(message::DefineDescriptor::SequenceModal(
                            message::SequenceModal::SequenceEdited(v.to_string()),
                        ))
                    }))
                    .width(Length::Units(500)),
                );
        }
    }

    card::simple(col.push(if sequence.valid {
        button::primary(None, "Apply")
            .on_press(Message::DefineDescriptor(
                message::DefineDescriptor::SequenceModal(message::SequenceModal::ConfirmSequence),
            ))
            .width(Length::Units(200))
    } else {
        button::primary(None, "Apply").width(Length::Units(200))
    }))
    .width(Length::Units(800))
    .into()
}

fn hw_list_view(
    i: usize,
    hw: &HardwareWallet,
    chosen: bool,
    processing: bool,
    selected: bool,
) -> Element<Message> {
    let mut bttn = Button::new(match hw {
        HardwareWallet::Supported {
            kind,
            version,
            fingerprint,
            alias,
            ..
        } => {
            if chosen && processing {
                hw::processing_hardware_wallet(kind, version.as_ref(), fingerprint, alias.as_ref())
            } else if selected {
                hw::selected_hardware_wallet(kind, version.as_ref(), fingerprint, alias.as_ref())
            } else {
                hw::supported_hardware_wallet(kind, version.as_ref(), fingerprint, alias.as_ref())
            }
        }
        HardwareWallet::Unsupported { version, kind, .. } => {
            hw::unsupported_hardware_wallet(&kind.to_string(), version.as_ref())
        }
    })
    .style(theme::Button::Secondary)
    .width(Length::Fill);
    if !processing && hw.is_supported() {
        bttn = bttn.on_press(Message::Select(i));
    }
    Container::new(bttn)
        .width(Length::Fill)
        .style(theme::Container::Card(theme::Card::Simple))
        .into()
}

pub fn backup_mnemonic<'a>(
    progress: (usize, usize),
    words: &'a [&'static str; 12],
    done: bool,
) -> Element<'a, Message> {
    layout(
        progress,
        Column::new()
            .push(text("Backup your mnemonic").bold().size(50))
            .push(text(prompt::MNEMONIC_HELP))
            .push(
                words
                    .iter()
                    .enumerate()
                    .fold(Column::new().spacing(5), |acc, (i, w)| {
                        acc.push(
                            Row::new()
                                .align_items(Alignment::End)
                                .push(
                                    Container::new(text(format!("#{}", i + 1)).small())
                                        .width(Length::Units(50)),
                                )
                                .push(text(*w).bold()),
                        )
                    }),
            )
            .push(checkbox(
                "I have backed up my mnemonic",
                done,
                Message::UserActionDone,
            ))
            .push(if done {
                button::primary(None, "Next")
                    .on_press(Message::Next)
                    .width(Length::Units(200))
            } else {
                button::primary(None, "Next").width(Length::Units(200))
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Alignment::Center),
    )
}

pub fn recover_mnemonic<'a>(
    progress: (usize, usize),
    words: &'a [(String, bool); 12],
    current: usize,
    suggestions: &'a Vec<String>,
    recover: bool,
    error: Option<&'a String>,
) -> Element<'a, Message> {
    layout(
        progress,
        Column::new()
            .push(text("Mnemonics import").bold().size(50))
            .push(text(prompt::RECOVER_MNEMONIC_HELP))
            .push_maybe(if recover {
                Some(
                    Column::new()
                        .align_items(Alignment::Center)
                        .push(
                            Container::new(if !suggestions.is_empty() {
                                suggestions.iter().fold(Row::new().spacing(5), |row, sugg| {
                                    row.push(
                                        Button::new(text(sugg))
                                            .style(theme::Button::Secondary)
                                            .on_press(Message::MnemonicWord(
                                                current,
                                                sugg.to_string(),
                                            )),
                                    )
                                })
                            } else {
                                Row::new()
                            })
                            // Fixed height in order to not move words list
                            .height(Length::Units(50)),
                        )
                        .push(words.iter().enumerate().fold(
                            Column::new().spacing(5),
                            |acc, (i, (word, valid))| {
                                acc.push(
                                    Row::new()
                                        .spacing(10)
                                        .align_items(Alignment::Center)
                                        .push(
                                            Container::new(text(format!("#{}", i + 1)).small())
                                                .width(Length::Units(50)),
                                        )
                                        .push(
                                            Container::new(TextInput::new("", word, move |msg| {
                                                Message::MnemonicWord(i, msg)
                                            }))
                                            .width(Length::Units(100)),
                                        )
                                        .push_maybe(if *valid {
                                            Some(
                                                icon::circle_check_icon()
                                                    .style(color::legacy::SUCCESS),
                                            )
                                        } else {
                                            None
                                        }),
                                )
                            },
                        ))
                        .push(Space::with_height(Length::Units(50)))
                        .push_maybe(
                            error.map(|e| card::invalid(text(e).style(color::legacy::ALERT))),
                        ),
                )
            } else {
                None
            })
            .push(if !recover {
                Row::new()
                    .spacing(10)
                    .push(
                        button::border(None, "Import mnemonic")
                            .on_press(Message::ImportMnemonic(true))
                            .width(Length::Units(200)),
                    )
                    .push(
                        button::primary(None, "Skip")
                            .on_press(Message::Skip)
                            .width(Length::Units(200)),
                    )
            } else {
                Row::new()
                    .spacing(10)
                    .push(
                        button::border(None, "Cancel")
                            .on_press(Message::ImportMnemonic(false))
                            .width(Length::Units(200)),
                    )
                    .push(
                        if words.iter().any(|(_, valid)| !valid) || error.is_some() {
                            button::primary(None, "Next").width(Length::Units(200))
                        } else {
                            button::primary(None, "Next")
                                .on_press(Message::Next)
                                .width(Length::Units(200))
                        },
                    )
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(100)
            .spacing(50)
            .align_items(Alignment::Center),
    )
}

fn layout<'a>(
    progress: (usize, usize),
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    Container::new(scrollable(
        Column::new()
            .push(
                Container::new(button::transparent(None, "< Previous").on_press(Message::Previous))
                    .padding(5),
            )
            .push(
                Container::new(text(format!("{}/{}", progress.0, progress.1)))
                    .width(Length::Fill)
                    .center_x(),
            )
            .push(Container::new(content).width(Length::Fill).center_x()),
    ))
    .center_x()
    .height(Length::Fill)
    .width(Length::Fill)
    .style(theme::Container::Background)
    .into()
}

mod threshsold_input {
    use iced::alignment::{self, Alignment};
    use iced::Length;
    use iced_lazy::{self, Component};
    use liana_ui::{component::text::*, icon, theme, widget::*};

    pub struct ThresholdInput<Message> {
        value: usize,
        max: usize,
        on_change: Box<dyn Fn(usize) -> Message>,
    }

    pub fn threshsold_input<Message>(
        value: usize,
        max: usize,
        on_change: impl Fn(usize) -> Message + 'static,
    ) -> ThresholdInput<Message> {
        ThresholdInput::new(value, max, on_change)
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        IncrementPressed,
        DecrementPressed,
    }

    impl<Message> ThresholdInput<Message> {
        pub fn new(
            value: usize,
            max: usize,
            on_change: impl Fn(usize) -> Message + 'static,
        ) -> Self {
            Self {
                value,
                max,
                on_change: Box::new(on_change),
            }
        }
    }

    impl<Message> Component<Message, iced::Renderer<theme::Theme>> for ThresholdInput<Message> {
        type State = ();
        type Event = Event;

        fn update(&mut self, _state: &mut Self::State, event: Event) -> Option<Message> {
            match event {
                Event::IncrementPressed => {
                    if self.value < self.max {
                        Some((self.on_change)(self.value.saturating_add(1)))
                    } else {
                        None
                    }
                }
                Event::DecrementPressed => {
                    if self.value > 1 {
                        Some((self.on_change)(self.value.saturating_sub(1)))
                    } else {
                        None
                    }
                }
            }
        }

        fn view(&self, _state: &Self::State) -> Element<Self::Event> {
            let button = |label, on_press| {
                Button::new(label)
                    .style(theme::Button::Transparent)
                    .width(Length::Units(50))
                    .on_press(on_press)
            };

            Column::new()
                .width(Length::Units(150))
                .push(button(icon::up_icon().size(30), Event::IncrementPressed))
                .push(text("Threshold:").small().bold())
                .push(
                    Container::new(text(format!("{}/{}", self.value, self.max)).size(30))
                        .align_y(alignment::Vertical::Center),
                )
                .push(button(icon::down_icon().size(30), Event::DecrementPressed))
                .align_items(Alignment::Center)
                .into()
        }
    }

    impl<'a, Message> From<ThresholdInput<Message>> for Element<'a, Message>
    where
        Message: 'a,
    {
        fn from(numeric_input: ThresholdInput<Message>) -> Self {
            iced_lazy::component(numeric_input)
        }
    }
}
