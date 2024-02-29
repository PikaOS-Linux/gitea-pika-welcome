// GTK crates
use adw::prelude::*;
use adw::*;
use glib::*;
//
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

// stack crates
mod community_page;
mod contribute_page;
mod look_and_feel_page;
mod recommended_addons_page;
mod setup_steps_page;
mod troubleshoot_page;
mod welcome_page;

use community_page::community_page;
use contribute_page::contribute_page;
use look_and_feel_page::look_and_feel_page;
use recommended_addons_page::recommended_addons_page;
use setup_steps_page::setup_steps_page;
use troubleshoot_page::troubleshoot_page;
use welcome_page::welcome_page;

use crate::config::{APP_GITHUB, APP_ICON, APP_ID, VERSION};

pub fn welcome_content_page(window: &adw::ApplicationWindow, content_box: &gtk::Box) {
    let glib_settings = gio::Settings::new(APP_ID);
    let internet_connected = Rc::new(RefCell::new(false));

    let (internet_loop_sender, internet_loop_receiver) = async_channel::unbounded();
    let internet_loop_sender = internet_loop_sender.clone();
    // The long running operation runs now in a separate thread
    gio::spawn_blocking(move || {
        loop {
            //match check_internet_connection() {
            //    Ok(_) => {
            //        internet_loop_sender.send_blocking(true).expect("The channel needs to be open.");
            //    }
            //    Err(_) => {
            //        internet_loop_sender.send_blocking(false).expect("The channel needs to be open.");
            //    }
            //}
            let check_internet_connection_cli = Command::new("ping")
                .arg("iso.pika-os.com")
                .arg("-c 1")
                .output()
                .expect("failed to execute process");
            if check_internet_connection_cli.status.success() {
                internet_loop_sender
                    .send_blocking(true)
                    .expect("The channel needs to be open.");
            } else {
                internet_loop_sender
                    .send_blocking(false)
                    .expect("The channel needs to be open.");
            }
        }
    });

    let window_banner = adw::Banner::builder().revealed(false).build();

    let window_title_bar = gtk::HeaderBar::builder().show_title_buttons(true).build();

    let credits_button = gtk::Button::builder()
        .icon_name("dialog-information-symbolic")
        .build();

    let mut dev_json_array: Vec<GString> = Vec::new();
    let dev_json_path = "/usr/share/pika-welcome/config/credits.json";
    let dev_json_data = std::fs::read_to_string(dev_json_path).expect("Unable to read json");
    let dev_json: serde_json::Value =
        serde_json::from_str(&dev_json_data).expect("JSON format invalid");
    if let serde_json::Value::Array(developers) = &dev_json["developers"] {
        for developer in developers {
            dev_json_array.push(developer["dev"].as_str().to_owned().unwrap().into())
        }
    }

    let mut translator_json_array: Vec<&str> = Vec::new();
    let translator_json_path = "/usr/share/pika-welcome/config/translators.json";
    let translator_json_data = std::fs::read_to_string(translator_json_path).expect("Unable to read json");
    let translator_json: serde_json::Value =
        serde_json::from_str(&translator_json_data).expect("JSON format invalid");
    if let serde_json::Value::Array(translators) = &translator_json["translators"] {
        for translator in translators {
            translator_json_array.push(translator["translator"].as_str().to_owned().unwrap())
        }
    }

    let credits_window = adw::AboutWindow::builder()
        .application_icon(APP_ICON)
        .application_name(t!("app_name"))
        .transient_for(window)
        .version(VERSION)
        .hide_on_close(true)
        .developer_name(t!("app_dev"))
        .developers(dev_json_array)
        .issue_url(APP_GITHUB.to_owned() + "/issues")
        .build();
    credits_window.add_credit_section(Some(&t!("translators").to_string()), &translator_json_array);

    content_box.append(&window_title_bar);

    let welcome_content_page_stack_box = gtk::Box::builder()
        .vexpand(true)
        .hexpand(true)
        .orientation(gtk::Orientation::Vertical)
        .build();

    let welcome_content_page_stack = gtk::Stack::builder()
        .vexpand(true)
        .hexpand(true)
        .transition_type(gtk::StackTransitionType::SlideUpDown)
        .build();

    let welcome_content_page_stack_sidebar = gtk::StackSidebar::builder()
        .vexpand(true)
        .hexpand(true)
        .stack(&welcome_content_page_stack)
        .build();

    let welcome_content_page_split_view = adw::OverlaySplitView::builder()
        .vexpand(true)
        .hexpand(true)
        .content(&welcome_content_page_stack_box)
        .sidebar(&welcome_content_page_stack_sidebar)
        .max_sidebar_width(300.0)
        .min_sidebar_width(300.0)
        .enable_hide_gesture(true)
        .enable_show_gesture(true)
        .build();

    let sidebar_toggle_button = gtk::ToggleButton::builder()
        .icon_name("view-right-pane-symbolic")
        .visible(false)
        .build();

    let startup_switch = gtk::CheckButton::builder()
        .label(t!("startup_switch_label"))
        .active(glib_settings.boolean("startup-show"))
        .build();

    let _sidebar_toggle_button_binding = welcome_content_page_split_view
        .bind_property("show_sidebar", &sidebar_toggle_button, "active")
        .sync_create()
        .bidirectional()
        .build();

    let welcome_content_page_split_view_breakpoint =
        adw::Breakpoint::new(BreakpointCondition::new_length(
            BreakpointConditionLengthType::MaxWidth,
            600.0,
            LengthUnit::Px,
        ));
    welcome_content_page_split_view_breakpoint.add_setter(
        &welcome_content_page_split_view,
        "collapsed",
        &true.to_value(),
    );
    welcome_content_page_split_view_breakpoint.add_setter(
        &startup_switch,
        "visible",
        &false.to_value(),
    );
    welcome_content_page_split_view_breakpoint.add_setter(
        &sidebar_toggle_button,
        "visible",
        &true.to_value(),
    );

    window.add_breakpoint(welcome_content_page_split_view_breakpoint);

    welcome_content_page_stack_box.append(&window_banner);
    welcome_content_page_stack_box.append(&welcome_content_page_stack);
    window_title_bar.pack_end(&credits_button);
    window_title_bar.pack_start(&sidebar_toggle_button);
    window_title_bar.pack_start(&startup_switch);
    content_box.append(&welcome_content_page_split_view);

    credits_button
        .connect_clicked(clone!(@weak credits_button => move |_| credits_window.present()));

    startup_switch.connect_toggled(clone!(@weak startup_switch => move |_| {
        let _ = glib_settings.set_boolean("startup-show", startup_switch.is_active());
    }));

    let internet_connected_status = internet_connected.clone();

    let internet_loop_context = MainContext::default();
    // The main loop executes the asynchronous block
    internet_loop_context.spawn_local(clone!(@weak window => async move {
        while let Ok(state) = internet_loop_receiver.recv().await {
            if state == true {
                *internet_connected_status.borrow_mut()=true;
            } else {
                *internet_connected_status.borrow_mut()=false;
            }
        }
    }));

    welcome_page(
        &welcome_content_page_stack,
        &window_banner,
        &internet_connected,
    );
    setup_steps_page(&welcome_content_page_stack, &window, &internet_connected);
    recommended_addons_page(&welcome_content_page_stack, &window, &internet_connected);
    look_and_feel_page(&welcome_content_page_stack, &window, &internet_connected);
    troubleshoot_page(&welcome_content_page_stack, &window, &internet_connected);
    community_page(&welcome_content_page_stack, &window, &internet_connected);
    contribute_page(&welcome_content_page_stack, &window, &internet_connected);
}
