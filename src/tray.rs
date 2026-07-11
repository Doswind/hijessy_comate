use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};

pub enum TrayAction {
    Capture,
    Exit,
}

pub struct TrayIconManager {
    _icon: TrayIcon,
    capture_id: MenuId,
    exit_id: MenuId,
}

impl TrayIconManager {
    pub fn new() -> anyhow::Result<Self> {
        let capture = MenuItem::new("截图", true, None);
        let settings = MenuItem::new("设置（即将支持）", false, None);
        let exit = MenuItem::new("退出", true, None);
        let capture_id = capture.id().clone();
        let exit_id = exit.id().clone();
        let menu = Menu::with_items(&[&capture, &settings, &exit])?;

        let icon = Icon::from_rgba(icon_rgba(), 16, 16)?;
        let tray = TrayIconBuilder::new()
            .with_tooltip("Hijessy")
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .build()?;

        Ok(Self {
            _icon: tray,
            capture_id,
            exit_id,
        })
    }

    pub fn poll(&self) -> Option<TrayAction> {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.capture_id {
                return Some(TrayAction::Capture);
            }
            if event.id == self.exit_id {
                return Some(TrayAction::Exit);
            }
        }

        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                button_state: tray_icon::MouseButtonState::Up,
                ..
            } = event
            {
                return Some(TrayAction::Capture);
            }
        }
        None
    }
}

fn icon_rgba() -> Vec<u8> {
    let mut pixels = vec![0; 16 * 16 * 4];
    for y in 2..14 {
        for x in 2..14 {
            let border = x == 2 || x == 13 || y == 2 || y == 13;
            let shutter = (5..=10).contains(&x) && (5..=10).contains(&y);
            if border || shutter {
                let i = (y * 16 + x) * 4;
                pixels[i] = 43;
                pixels[i + 1] = 127;
                pixels[i + 2] = 255;
                pixels[i + 3] = 255;
            }
        }
    }
    pixels
}
