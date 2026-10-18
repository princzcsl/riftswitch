//! Fenêtre de défi hCaptcha, résolu par l'utilisateur (d'après League-Account-Manager).
//!
//! Riot protège la connexion par un hCaptcha « entreprise » lié à l'origine `authenticate.riotgames.com`.
//! Pour que le jeton résolu par l'utilisateur soit accepté, le widget doit tourner sur cette origine :
//! on navigue donc vers `https://authenticate.riotgames.com/challenge.html` et on sert UNIQUEMENT ce
//! document depuis une page locale (interception de la requête document). Toutes les autres requêtes
//! du widget partent vers les vrais serveurs de Riot, exactement comme le client officiel.
//!
//! C'est l'utilisateur qui résout le défi ; RiftSwitch ne fait qu'afficher la fenêtre et récupérer
//! le jeton. Aucune valeur de jeton n'est journalisée.

use crate::error::AppResult;

/// Affiche la fenêtre de défi et renvoie le jeton résolu, ou `None` si l'utilisateur l'a fermée.
/// Bloquant (boucle de messages Windows) : appeler via `spawn_blocking`.
#[cfg(windows)]
pub fn solve(site_key: &str, rq_data: &str, host: &str, user_data_dir: &std::path::Path) -> AppResult<Option<String>> {
    imp::solve(site_key, rq_data, host, user_data_dir)
}

#[cfg(not(windows))]
pub fn solve(_: &str, _: &str, _: &str, _: &std::path::Path) -> AppResult<Option<String>> {
    Err(crate::error::AppError::code("captcha_unsupported"))
}

/// Page locale servie sur l'origine de l'authentificateur : charge hCaptcha et renvoie le jeton.
pub fn build_html(site_key: &str, rq_data: &str, host: &str) -> String {
    let site_key = serde_json::to_string(site_key).unwrap_or_else(|_| "\"\"".into());
    let rq_data = serde_json::to_string(rq_data).unwrap_or_else(|_| "\"\"".into());
    let host_json = serde_json::to_string(host).unwrap_or_else(|_| "\"\"".into());
    let hl = "en-US";
    let script = format!(
        "https://js.hcaptcha.com/1/api.js?render=explicit&onload=hcaptchaOnLoad&hl={hl}&recaptchacompat=off&host={}",
        urlencode(host)
    );
    format!(
        r#"<!DOCTYPE html><html><head><meta charset='utf-8'>
<style>body{{background:#12131f;color:#e9eafc;font-family:Segoe UI,sans-serif;margin:0;padding:18px;display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:calc(100vh - 36px);box-sizing:border-box}}#status{{margin-bottom:14px;font-size:14px;opacity:.85}}#checkbox{{min-width:303px;min-height:78px}}</style>
<script>function hcaptchaOnLoad(){{window.__ready=true;}}</script>
<script src='{script}'></script></head><body>
<div id='status'>Chargement du défi…</div><div id='checkbox'></div>
<script>
function post(m){{try{{window.chrome.webview.postMessage(m);}}catch(e){{}}}}
function go(){{
  if(!(window.__ready&&typeof hcaptcha!=='undefined')){{setTimeout(go,100);return;}}
  document.getElementById('status').textContent='Résolution du défi…';
  try{{
    var opts={{sitekey:{site_key},size:'invisible',hl:'{hl}',
      'error-callback':function(c){{post({{type:'err',error:String(c)}});}},
      'chalexpired-callback':function(){{document.getElementById('status').textContent='Défi expiré.';}},
      callback:function(t){{if(t)post({{type:'ok',token:t}});}}}};
    if({host_json}!=='')opts.host={host_json};
    var id=hcaptcha.render('checkbox',opts);
    hcaptcha.execute(id,{{rqdata:{rq_data},async:true}}).then(function(r){{
      var t=(r&&typeof r==='object')?r.response:r;
      if(!t){{post({{type:'err',error:'empty'}});return;}}
      post({{type:'ok',token:t}});
      document.getElementById('status').textContent='Défi résolu.';
    }}).catch(function(e){{post({{type:'err',error:String(e)}});document.getElementById('status').textContent='Échec du défi.';}});
  }}catch(ex){{post({{type:'err',error:String(ex)}});document.getElementById('status').textContent='Erreur : '+String(ex);}}
}}
go();
</script></body></html>"#
    )
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(windows)]
mod imp {
    use super::build_html;
    use crate::error::{AppError, AppResult};
    use crate::util::log;
    use std::cell::RefCell;
    use std::sync::mpsc;
    use webview2_com::Microsoft::Web::WebView2::Win32::*;
    use webview2_com::{
        CreateCoreWebView2ControllerCompletedHandler,
        CreateCoreWebView2EnvironmentCompletedHandler, WebMessageReceivedEventHandler,
        WebResourceRequestedEventHandler,
    };
    use windows::core::{w, HSTRING, PCWSTR, PWSTR};
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Shell::SHCreateMemStream;
    use windows::Win32::UI::WindowsAndMessaging::*;

    thread_local! {
        /// Jeton résolu (partagé entre les handlers WebView2 et la boucle de messages).
        static RESULT: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        match msg {
            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }

    pub fn solve(
        site_key: &str,
        rq_data: &str,
        host: &str,
        user_data_dir: &std::path::Path,
    ) -> AppResult<Option<String>> {
        RESULT.with(|r| *r.borrow_mut() = None);
        unsafe { run(site_key, rq_data, host, user_data_dir) }
    }

    unsafe fn run(
        site_key: &str,
        rq_data: &str,
        host: &str,
        user_data_dir: &std::path::Path,
    ) -> AppResult<Option<String>> {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let result = (|| -> AppResult<Option<String>> {
            let hinstance = GetModuleHandleW(None).map_err(com)?;
            let class = w!("RiftSwitchChallenge");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                lpszClassName: class,
                hInstance: hinstance.into(),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                ..Default::default()
            };
            RegisterClassW(&wc);

            let (w, h) = (460, 680);
            let sw = GetSystemMetrics(SM_CXSCREEN);
            let sh = GetSystemMetrics(SM_CYSCREEN);
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class,
                w!("Riot — vérification"),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
                (sw - w) / 2,
                (sh - h) / 2,
                w,
                h,
                None,
                None,
                Some(hinstance.into()),
                None,
            )
            .map_err(com)?;

            // Environnement WebView2 (dossier de données dédié : évite d'écrire à côté de l'exe).
            let folder = HSTRING::from(user_data_dir.as_os_str());
            let environment = {
                let (tx, rx) = mpsc::channel();
                CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
                    Box::new(move |handler| {
                        CreateCoreWebView2EnvironmentWithOptions(
                            PCWSTR::null(),
                            PCWSTR(folder.as_ptr()),
                            None,
                            &handler,
                        )
                        .map_err(webview2_com::Error::WindowsError)
                    }),
                    Box::new(move |code, env| {
                        code?;
                        tx.send(env).expect("env");
                        Ok(())
                    }),
                )
                .map_err(wv)?;
                rx.recv().ok().flatten().ok_or_else(|| AppError::code("captcha_env"))?
            };

            // Contrôleur attaché à la fenêtre.
            let controller = {
                let (tx, rx) = mpsc::channel();
                let env = environment.clone();
                CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
                    Box::new(move |handler| {
                        env.CreateCoreWebView2Controller(hwnd, &handler)
                            .map_err(webview2_com::Error::WindowsError)
                    }),
                    Box::new(move |code, controller| {
                        code?;
                        tx.send(controller).expect("controller");
                        Ok(())
                    }),
                )
                .map_err(wv)?;
                rx.recv().ok().flatten().ok_or_else(|| AppError::code("captcha_ctrl"))?
            };

            let mut rect = RECT::default();
            GetClientRect(hwnd, &mut rect).map_err(com)?;
            controller.SetBounds(rect).map_err(com)?;
            controller.SetIsVisible(true).map_err(com)?;
            let webview = controller.CoreWebView2().map_err(com)?;

            // Servir notre page uniquement pour le document de l'origine de l'authentificateur.
            let page_url = format!("https://{host}/challenge.html");
            let filter = HSTRING::from(page_url.as_str());
            webview
                .AddWebResourceRequestedFilter(
                    PCWSTR(filter.as_ptr()),
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT,
                )
                .map_err(com)?;

            let html = build_html(site_key, rq_data, host);
            let env_for_handler = environment.clone();
            let mut token = 0;
            webview
                .add_WebResourceRequested(
                    &WebResourceRequestedEventHandler::create(Box::new(move |_wv, args| {
                        if let Some(args) = args {
                            let bytes = html.as_bytes();
                            let stream = SHCreateMemStream(Some(bytes));
                            if let Some(stream) = stream {
                                let headers = w!("Content-Type: text/html; charset=utf-8");
                                if let Ok(resp) = env_for_handler.CreateWebResourceResponse(
                                    &stream,
                                    200,
                                    w!("OK"),
                                    headers,
                                ) {
                                    let _ = args.SetResponse(&resp);
                                }
                            }
                        }
                        Ok(())
                    })),
                    &mut token,
                )
                .map_err(com)?;

            // Récupérer le jeton résolu.
            let mut msg_token = 0;
            webview
                .add_WebMessageReceived(
                    &WebMessageReceivedEventHandler::create(Box::new(move |_wv, args| {
                        if let Some(args) = args {
                            let mut json = PWSTR::null();
                            if args.WebMessageAsJson(&mut json).is_ok() {
                                let s = webview2_com::take_pwstr(json);
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                                    if v.get("type").and_then(|x| x.as_str()) == Some("ok") {
                                        if let Some(t) = v.get("token").and_then(|x| x.as_str()) {
                                            RESULT.with(|r| *r.borrow_mut() = Some(t.to_string()));
                                            unsafe { PostQuitMessage(0) };
                                        }
                                    }
                                }
                            }
                        }
                        Ok(())
                    })),
                    &mut msg_token,
                )
                .map_err(com)?;

            let nav = HSTRING::from(page_url.as_str());
            webview.Navigate(&nav).map_err(com)?;

            let _ = ShowWindow(hwnd, SW_SHOW);

            // Boucle de messages : se termine quand le jeton arrive (PostQuitMessage) ou à la fermeture.
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = controller.Close();
            if IsWindow(Some(hwnd)).as_bool() {
                let _ = DestroyWindow(hwnd);
            }
            let token = RESULT.with(|r| r.borrow_mut().take());
            let _ = &nav;
            let _ = &filter;
            Ok(token)
        })();
        CoUninitialize();
        if let Err(e) = &result {
            log(format!("captcha: échec {} {}", e.code, e.detail));
        }
        result
    }

    fn com(e: windows::core::Error) -> AppError {
        AppError::new("captcha", e.message())
    }
    fn wv(e: webview2_com::Error) -> AppError {
        AppError::new("captcha", format!("{e:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::build_html;

    #[test]
    fn html_embeds_sitekey_and_host() {
        let html = build_html("SITEKEY", "RQDATA", "authenticate.riotgames.com");
        assert!(html.contains("\"SITEKEY\""));
        assert!(html.contains("\"RQDATA\""));
        assert!(html.contains("host=authenticate.riotgames.com"));
        assert!(html.contains("hcaptcha.execute"));
    }
}
