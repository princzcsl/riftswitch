//! Fenêtre du Riot Client : une fois le jeu ouvert, on la ferme comme avec la croix (`WM_CLOSE`).
//! Le client continue de tourner en arrière-plan (zone de notification) ; aucun processus n'est arrêté.

/// Ferme les fenêtres principales visibles du Riot Client. Renvoie leur nombre.
#[cfg(windows)]
pub fn close_riot_client_windows() -> usize {
    use windows::core::BOOL;
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindow, GetWindowThreadProcessId, IsWindowVisible, PostMessageW, GW_OWNER,
        WM_CLOSE,
    };

    struct Ctx {
        pids: Vec<u32>,
        found: Vec<isize>,
    }

    unsafe extern "system" fn collect(hwnd: HWND, l: LPARAM) -> BOOL {
        let ctx = &mut *(l.0 as *mut Ctx);
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        // Fenêtres de premier niveau visibles et sans propriétaire : la fenêtre principale du client.
        let top_level = GetWindow(hwnd, GW_OWNER).is_err();
        if ctx.pids.contains(&pid) && IsWindowVisible(hwnd).as_bool() && top_level {
            ctx.found.push(hwnd.0 as isize);
        }
        true.into()
    }

    let pids = super::process::riot_client_ui_pids();
    if pids.is_empty() {
        return 0;
    }
    let mut ctx = Ctx {
        pids,
        found: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut ctx as *mut Ctx as isize));
        for raw in &ctx.found {
            let hwnd = HWND(*raw as *mut core::ffi::c_void);
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
    ctx.found.len()
}

#[cfg(not(windows))]
pub fn close_riot_client_windows() -> usize {
    0
}
