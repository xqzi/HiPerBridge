use druid::{ widget::Flex, * };

use scl_gui_widgets::{
    theme::icons::{ IconColorKey, IconKeyPair, IconPathKey },
    widgets::*,
    WidgetExt as _,
};

use std::{ fmt::Write, time::Duration };

use crate::{
    app_state::AppState,
    hiper::{ get_hiper_dir, run_hiper_in_thread, stop_hiper },
    open_url::open_url,
};

pub const CLIPBOARD_TEXT_ICON: IconKeyPair = (
    CLIPBOARD_TEXT_PATH,
    CLIPBOARD_TEXT_COLOR,
    CLIPBOARD_TEXT_COLOR,
);
pub const CLIPBOARD_TEXT_COLOR: IconColorKey = IconColorKey::new("clipboard-text-color");
pub const CLIPBOARD_TEXT_PATH: IconPathKey = IconPathKey::new("clipboard-text-path");

pub const SET_START_TEXT: Selector<&str> = Selector::new("set-start-text");
pub const SET_IP: Selector<String> = Selector::new("set-ip");
pub const SET_VALID: Selector<String> = Selector::new("set-valid");
pub const SET_WARNING: Selector<String> = Selector::new("set-warning");
pub const SET_DISABLED: Selector<bool> = Selector::new("set-disabled");
pub const REQUEST_RESTART: Selector = Selector::new("request-restart");
pub const SHOW_HIPER_WINDOW: Selector = Selector::new("show-hiper-window");

fn main_page() -> Box<dyn Widget<AppState>> {
    Flex::column()
        // .with_child(label::new("HiPer Bridge").with_font(typography::SUBHEADER))
        .with_child(label::new("本联机模块与PMCL互联互通"))
        .with_child(label::new("房主与加入方的暗号必须一致"))
        .with_child(label::new("暗号越长越复杂越安全越不会重复被误入"))
        .with_spacer(10.0)
        .with_child(label::new("永久免费 官网:pmcl.fun"))
        .with_flex_child(
            label
                ::dynamic(|data: &AppState, _| data.warning.to_owned())
                .with_text_color(Color::Rgba32(0x9d5d00ff))
                .scroll()
                .vertical()
                .expand(),
            1.0
        )
        .with_child(
            Flex::row()
                .with_flex_child(
                    label
                        ::dynamic(|data: &AppState, _| {
                            if data.ip.is_empty() {
                                "".into()
                            } else {
                                let sec = data.run_time % 60;
                                let min = data.run_time / 60;
                                let hour = min / 60;
                                let day = hour / 24;
                                let min = min % 60;
                                let hour = hour % 24;

                                let mut run_time_formated = String::with_capacity(16);

                                if day > 0 {
                                    let _ = write!(run_time_formated, "{}:", day);
                                }

                                if day > 0 || hour > 0 {
                                    let _ = write!(run_time_formated, "{:02}:", hour);
                                }

                                let _ = write!(run_time_formated, "{:02}:{:02}", min, sec);

                                format!(
                                    "联机暗号: {}\n网络地址: {}\n运行时间: {}",
                                    data.token,
                                    data.ip,
                                    run_time_formated
                                )
                            }
                        })
                        .with_text_color(Color::Rgba32(0x0f7b0fff))
                        .expand_width(),
                    1.0
                )
                .with_child(
                    IconButton::new(CLIPBOARD_TEXT_ICON)
                        .with_flat(true)
                        .on_click(|_, data: &mut AppState, _| {
                            use clipboard::ClipboardProvider;
                            #[cfg(windows)]
                            {
                                if
                                    let Ok(mut cb) =
                                        clipboard::windows_clipboard::WindowsClipboardContext::new()
                                {
                                    let _ = cb.set_contents(
                                        format!(
                                            "我正在使用PMCL邀请你加入到我的世界\n\n连接地址是 {} \n请使用联机暗号 {}\n通过PMCL加入游戏\n\n联机启动器下载地址为 pmcl.fun",
                                            data.ip,
                                            data.token
                                        )
                                    );
                                }
                            }
                            #[cfg(target_os = "linux")]
                            {
                                if
                                    let Ok(mut cb) =
                                        clipboard::x11_clipboard::X11ClipboardContext::<clipboard::x11_clipboard::Clipboard>::new()
                                {
                                    let _ = cb.set_contents(
                                        format!(
                                            "我正在使用PMCL邀请你加入到我的世界\n\n连接地址是 {} \n请使用联机暗号 {}\n通过PMCL加入游戏\n\n联机启动器下载地址为 pmcl.fun",
                                            data.ip,
                                            data.token
                                        )
                                    );
                                }
                            }
                            #[cfg(target_os = "macos")]
                            {
                                if
                                    let Ok(mut cb) =
                                        clipboard::osx_clipboard::OSXClipboardContext::new()
                                {
                                    let _ = cb.set_contents(
                                        format!(
                                            "我正在使用PMCL邀请你加入到我的世界\n\n连接地址是 {} \n请使用联机暗号 {}\n通过PMCL加入游戏\n\n联机启动器下载地址为 pmcl.fun",
                                            data.ip,
                                            data.token
                                        )
                                    );
                                }
                            }
                        })
                )
                .cross_axis_alignment(widget::CrossAxisAlignment::End)
                .show_if(|data: &AppState, _| !data.ip.is_empty())
        )
        .with_child(
            label
                ::new("联机暗号")
                .show_if(|data: &AppState, _| data.ip.is_empty())
                .padding((0.0, 5.0))
        )
        .with_child(
            PasswordBox::new()
                .lens(AppState::token)
                .show_if(|data, _| data.ip.is_empty())
        )
        .with_spacer(10.0)
        .with_child(
            Flex::row()
                .with_flex_child(
                    Button::dynamic(|data: &AppState, _| data.start_button.to_owned())
                        .with_accent(true)
                        .on_click(|ctx, data, _| {
                            let ctx = ctx.get_external_handle();
                            let token = data.token.to_owned();
                            match data.start_button {
                                "启动" => {
                                    run_hiper_in_thread(
                                        ctx,
                                        token,
                                        data.use_tun,
                                        data.use_tcp,
                                        data.use_igmp,
                                        data.fast_mode,
                                        data.debug_mode,
                                        data.kill_hiper_when_start
                                    );
                                }
                                "返回" => {
                                    std::thread::spawn(move || {
                                        let _ = ctx.submit_command(
                                            SET_DISABLED,
                                            true,
                                            Target::Auto
                                        );
                                        stop_hiper(ctx.to_owned());
                                        let _ = ctx.submit_command(
                                            SET_DISABLED,
                                            false,
                                            Target::Auto
                                        );
                                    });
                                }
                                _ => {
                                    println!(
                                        "Warning: Unknown start button text {}",
                                        data.start_button
                                    );
                                }
                            }
                        })
                        .expand_width()
                        .disabled_if(|data: &AppState, _| data.token.trim().is_empty()),
                    1.0
                )
                .must_fill_main_axis(true)
        )
        // .must_fill_main_axis(true)
        .cross_axis_alignment(widget::CrossAxisAlignment::Fill)
        .padding((10.0, 10.0))
        .boxed()
}

#[cfg(target_os = "macos")]
fn mac_init() -> Box<dyn Widget<AppState>> {
    Flex::column()
        .with_child(label::new("提权初始化提示").with_text_size(16.0))
        .with_spacer(10.0)
        .with_child(
            label::new(
                "由于 MacOS 严格的权限体系，HiPer Bridge 需要管理员权限注册 HiPer 系统网络服务，若出现验证提示，请通过验证允许。"
            )
        )
        .with_flex_spacer(1.0)
        .with_flex_child(
            label
                ::dynamic(|data: &AppState, _| data.init_message.to_owned())
                .scroll()
                .vertical()
                .expand(),
            1.0
        )
        .with_flex_spacer(1.0)
        .with_child(
            Button::new("允许并授权初始化 HiPer Bridge")
                .with_accent(true)
                .on_click(|ctx, _, _| {
                    let ctx = ctx.get_external_handle();
                    std::thread::spawn(move || {
                        if let Err(err) = crate::mac::install_hiper(ctx.to_owned()) {
                            ctx.add_idle_callback(move |data: &mut AppState| {
                                data.init_message = format!("安装出错，请重试：\n{}", err);
                                data.running_script = false;
                            });
                        } else {
                            let _ = ctx.submit_command(QUERY_POP_PAGE, "", Target::Auto);
                            let _ = ctx.submit_command(ENABLE_BACK_PAGE, true, Target::Auto);
                            ctx.add_idle_callback(|data: &mut AppState| {
                                data.init_message = "".into();
                                data.running_script = false;
                            });
                        }
                    });
                })
                .disabled_if(|data: &AppState, _| data.running_script)
        )
        .cross_axis_alignment(widget::CrossAxisAlignment::Fill)
        .padding((10.0, 10.0))
        .expand()
        .boxed()
}

pub struct AppWrapper {
    inner: WidgetPod<AppState, Box<dyn Widget<AppState>>>,
    run_timer: TimerToken,
}

impl AppWrapper {
    pub fn new(inner: impl Widget<AppState> + 'static) -> Self {
        Self {
            inner: WidgetPod::new(inner).boxed(),
            run_timer: TimerToken::INVALID,
        }
    }
}

impl Widget<AppState> for AppWrapper {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        if let Event::Timer(tt) = event {
            if &self.run_timer == tt {
                data.run_time += 1;
                self.run_timer = ctx.request_timer(Duration::from_secs(1));
                ctx.request_update();
            }
        } else if let Event::Command(cmd) = event {
            if let Some(ip) = cmd.get(SET_IP) {
                if ip.is_empty() {
                    self.run_timer = TimerToken::INVALID;
                } else {
                    data.run_time = 0;
                    self.run_timer = ctx.request_timer(Duration::from_secs(1));
                }
            }
        } else if let Event::WindowConnected = event {
            #[cfg(target_os = "macos")]
            {
                if !crate::mac::check_sudoer(&crate::mac::get_current_user()) {
                    ctx.submit_command(PUSH_PAGE.with("mac-init"));
                    ctx.submit_command(ENABLE_BACK_PAGE.with(false));
                }
            }
        }
        self.inner.event(ctx, event, data, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.run_timer = ctx.request_timer(Duration::from_secs(1));
        }
        self.inner.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &AppState, data: &AppState, env: &Env) {
        self.inner.update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &AppState,
        env: &Env
    ) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        self.inner.paint(ctx, data, env)
    }
}

pub fn ui_builder() -> impl Widget<AppState> {
    AppWrapper::new({
        let mut pager = PageSwitcher::new();
        pager.add_page("main", Box::new(main_page));
        #[cfg(target_os = "macos")]
        {
            pager.add_page("mac-init", Box::new(mac_init));
        }
        pager
    })
}
