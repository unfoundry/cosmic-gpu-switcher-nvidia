// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{window::Id, Limits, Subscription, Alignment};
use cosmic::prelude::*;
use cosmic::widget;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
    current_mode: String,
    status: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    UpdateConfig(Config),
    GraphicsModeFetched(Result<String, String>),
    SetGraphicsMode(String),
    GraphicsModeChanged(Result<(), String>),
}

fn get_resource_path(relative_path: &str) -> PathBuf {
    let system_path = PathBuf::from(format!("/usr/share/cosmic-gpu-switcher-nvidia/resources/{}", relative_path));
    if system_path.exists() {
        system_path
    } else {
        PathBuf::from("resources").join(relative_path)
    }
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "io.github.unfoundry.cosmic-gpu-switcher-nvidia";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            current_mode: "loading...".to_string(),
            ..Default::default()
        };

        let init_task = Task::perform(
            async {
                match tokio::task::spawn_blocking(|| {
                    Command::new("system76-power").arg("graphics").output()
                }).await {
                    Ok(Ok(output)) if output.status.success() => {
                        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                    }
                    Ok(Ok(output)) => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
                    Ok(Err(e)) => Err(e.to_string()),
                    Err(e) => Err(e.to_string()),
                }
            },
            Message::GraphicsModeFetched,
        )
        .map(Into::into);

        (app, init_task)
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("display-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let mut content = widget::Column::new()
            .spacing(0)
            .padding(0)
            .width(cosmic::iced::Length::Fill);

        // Dynamically choose current icon based on active mode
        let mode_lower = self.current_mode.to_lowercase();
        let current_icon_file = if mode_lower.contains("integrated") {
            "cpu.svg"
        } else if mode_lower.contains("compute") {
            "compute.svg"
        } else if mode_lower.contains("hybrid") {
            "hybrid.svg"
        } else if mode_lower.contains("nvidia") {
            "nvidia.svg"
        } else {
            "cpu.svg"
        };

        let current_svg = widget::svg(get_resource_path(&format!("icons/{}", current_icon_file)))
            .width(35)
            .height(30);


        // Top Header matching system applet style
        let header_text_col = widget::Column::new()
            .spacing(2)
            .push(widget::text::body("Graphics Mode").size(13))
            .push(
                widget::text::caption(if self.status.is_empty() { &self.current_mode } else { &self.status })
                    .size(11)
            );

        let top_row = widget::Row::new()
            .spacing(14)
            .align_y(Alignment::Center)
            .push(current_svg)
            .push(header_text_col);

        let header_container = widget::container(top_row).padding(12);
        content = content.push(header_container);

        // Visible native divider line spanning full width
        let divider = cosmic::iced::widget::rule::horizontal(1);
        content = content.push(divider);

        let is_int = self.current_mode.eq_ignore_ascii_case("integrated");
        let is_cmp = self.current_mode.eq_ignore_ascii_case("compute");
        let is_hyb = self.current_mode.eq_ignore_ascii_case("hybrid");
        let is_nvd = self.current_mode.eq_ignore_ascii_case("nvidia");

        // --- 1. Integrated Button ---
        let mut row_int = widget::Row::new()
            .spacing(14)
            .align_y(Alignment::Center)
            .width(cosmic::iced::Length::Fill)
            .push(widget::svg(get_resource_path("icons/cpu.svg")).width(22).height(22))
            .push(
                widget::Column::new()
                    .spacing(2)
                    .push(widget::text::body("Integrated").size(13))
                    .push(widget::text::caption("Uses CPU graphics to maximize battery life.").size(11))
                    .width(cosmic::iced::Length::Fill)
            );
        if is_int {
            row_int = row_int.push(
                widget::container(
                    widget::icon::from_name("emblem-ok-symbolic")
                        .size(12)
                        .symbolic(true)
                )
            );
        } else {
            row_int = row_int.push(widget::container(widget::space::horizontal().width(1.0)));
        }
        let btn_int = cosmic::applet::menu_button(row_int)
            .on_press(Message::SetGraphicsMode("integrated".into()));

        // --- 2. Compute Button ---
        let mut row_cmp = widget::Row::new()
            .spacing(14)
            .align_y(Alignment::Center)
            .width(cosmic::iced::Length::Fill)
            .push(widget::svg(get_resource_path("icons/compute.svg")).width(22).height(22))
            .push(
                widget::Column::new()
                    .spacing(2)
                    .push(widget::text::body("Compute").size(13))
                    .push(widget::text::caption("NVIDIA active for CUDA, displays use Integrated.").size(11))
                    .width(cosmic::iced::Length::Fill)
            );
        if is_cmp {
            row_cmp = row_cmp.push(
                widget::container(
                    widget::icon::from_name("emblem-ok-symbolic")
                        .size(12)
                        .symbolic(true)
                )
            );
        } else {
            row_cmp = row_cmp.push(widget::container(widget::space::horizontal().width(1.0)));
        }
        let btn_cmp = cosmic::applet::menu_button(row_cmp)
            .on_press(Message::SetGraphicsMode("compute".into()));

        // --- 3. Hybrid Button ---
        let mut row_hyb = widget::Row::new()
            .spacing(14)
            .align_y(Alignment::Center)
            .width(cosmic::iced::Length::Fill)
            .push(widget::svg(get_resource_path("icons/hybrid.svg")).width(22).height(22))
            .push(
                widget::Column::new()
                    .spacing(2)
                    .push(widget::text::body("Hybrid").size(13))
                    .push(widget::text::caption("Auto-switch between integrated and NVIDIA.").size(11))
                    .width(cosmic::iced::Length::Fill)
            );
        if is_hyb {
            row_hyb = row_hyb.push(
                widget::container(
                    widget::icon::from_name("emblem-ok-symbolic")
                        .size(12)
                        .symbolic(true)
                )
            );
        } else {
            row_hyb = row_hyb.push(widget::container(widget::space::horizontal().width(1.0)));
        }
        let btn_hyb = cosmic::applet::menu_button(row_hyb)
            .on_press(Message::SetGraphicsMode("hybrid".into()));

        // --- 4. NVIDIA Button ---
        let mut row_nvd = widget::Row::new()
            .spacing(14)
            .align_y(Alignment::Center)
            .width(cosmic::iced::Length::Fill)
            .push(widget::svg(get_resource_path("icons/nvidia.svg")).width(22).height(22))
            .push(
                widget::Column::new()
                    .spacing(2)
                    .push(widget::text::body("NVIDIA").size(13))
                    .push(widget::text::caption("Force high-performance NVIDIA rendering.").size(11))
                    .width(cosmic::iced::Length::Fill)
            );
        if is_nvd {
            row_nvd = row_nvd.push(
                widget::container(
                    widget::icon::from_name("emblem-ok-symbolic")
                        .size(12)
                        .symbolic(true)
                )
            );
        } else {
            row_nvd = row_nvd.push(widget::container(widget::space::horizontal().width(1.0)));
        }
        let btn_nvd = cosmic::applet::menu_button(row_nvd)
            .on_press(Message::SetGraphicsMode("nvidia".into()));

        // Vertical list column spanning full width with edge-to-edge menu buttons
        let list_col = widget::Column::new()
            .spacing(0)
            .width(cosmic::iced::Length::Fill)
            .push(btn_int)
            .push(btn_cmp)
            .push(btn_hyb)
            .push(btn_nvd);

        content = content.push(list_col);

        self.core.applet.popup_container(content).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        self.core()
            .watch_config::<Config>(Self::APP_ID)
            .map(|update| Message::UpdateConfig(update.config))
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(380.0)
                        .min_width(320.0)
                        .min_height(180.0)
                        .max_height(440.0);

                    let refresh = Task::perform(
                        async {
                            match tokio::task::spawn_blocking(|| {
                                Command::new("system76-power").arg("graphics").output()
                            }).await {
                                Ok(Ok(output)) if output.status.success() => {
                                    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                                }
                                Ok(Ok(output)) => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
                                Ok(Err(e)) => Err(e.to_string()),
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::GraphicsModeFetched,
                    )
                    .map(Into::into);

                    let popup_task = get_popup(popup_settings);
                    return Task::batch(vec![popup_task, refresh]);
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::GraphicsModeFetched(res) => {
                match res {
                    Ok(mode) => {
                        self.current_mode = mode;
                        self.status.clear();
                    }
                    Err(e) => {
                        self.status = format!("Err: {}", e);
                    }
                }
            }
            Message::SetGraphicsMode(mode) => {
                self.status = format!("Applying {}", mode);
                let mode_clone = mode.clone();
                return Task::perform(
                    async move {
                        match tokio::task::spawn_blocking(move || {
                            Command::new("system76-power")
                                // .arg("system76-power")
                                .arg("graphics")
                                .arg(&mode_clone)
                                .status()
                        }).await {
                            Ok(Ok(status)) if status.success() => Ok(()),
                            Ok(Ok(status)) => Err(format!("Code {}", status)),
                            Ok(Err(e)) => Err(e.to_string()),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::GraphicsModeChanged,
                )
                .map(Into::into);
            }
            Message::GraphicsModeChanged(res) => {
                match res {
                    Ok(()) => {
                        self.status = "Reboot required".into();
                        return Task::perform(
                            async {
                                match tokio::task::spawn_blocking(|| {
                                    Command::new("system76-power").arg("graphics").output()
                                }).await {
                                    Ok(Ok(output)) if output.status.success() => {
                                        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                                    }
                                    Ok(Ok(output)) => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
                                    Ok(Err(e)) => Err(e.to_string()),
                                    Err(e) => Err(e.to_string()),
                                }
                            },
                            Message::GraphicsModeFetched,
                        )
                        .map(Into::into);
                    }
                    Err(e) => {
                        self.status = format!("Failed: {}", e);
                    }
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}