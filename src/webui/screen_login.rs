use crate::types::AppError;
use crate::webui::LogoSvg;
use leptos::{form::ActionForm, prelude::*};

#[component]
pub fn LoginScreen() -> impl IntoView {
    let login_action = ServerAction::<Login>::new();
    let pending = login_action.pending();
    let action_value = login_action.value();

    let has_success = move || matches!(action_value.get().as_ref(), Some(Ok(_)));
    let error_message = move || {
        action_value
            .get()
            .as_ref()
            .and_then(|res| res.as_ref().err())
            .map(|err| err.message().to_string())
    };

    // handle client-side refresh after successful login
    Effect::watch(
        move || action_value.get(),
        move |result, _, _| {
            if matches!(result.as_ref(), Some(Ok(_))) {
                // force a full page reload to ensure session is properly loaded
                let _ = window().location().set_href("/");
            }
        },
        false,
    );

    view! {
        <div
            class="min-h-screen relative flex items-center justify-center px-4 py-10 overflow-hidden font-sans text-stone-200 selection:bg-orange-500/30"
            style="background-color: #0c0c0e; background-image: linear-gradient(to bottom, #161618 0%, #0c0c0e 100%);"
        >
            <div
                class="absolute inset-0 opacity-[0.03] pointer-events-none mix-blend-overlay"
                style="background-image: url('data:image/svg+xml,%3Csvg viewBox=\'0 0 200 200\' xmlns=\'http://www.w3.org/2000/svg\'%3E%3Cfilter id=\'noiseFilter\'%3E%3CfeTurbulence type=\'fractalNoise\' baseFrequency=\'0.8\' numOctaves=\'3\' stitchTiles=\'stitch\'/%3E%3C/filter%3E%3Crect width=\'100%25\' height=\'100%25\' filter=\'url(%23noiseFilter)\'/%3E%3C/svg%3E');"
            ></div>

            <div class="relative w-full max-w-md space-y-8 z-10">
                <div class="text-center">
                    <LogoSvg size=16 class="mx-auto mb-4 opacity-90 drop-shadow-lg" />
                    <h1 class="text-3xl font-bold tracking-tight mb-2 text-white">
                        "Sign in to Bento"
                    </h1>
                    <p class="text-stone-400">"Enter your username and password to continue."</p>
                </div>

                <ActionForm action=login_action>
                    <div class="bg-[#18181b] border-t border-white/10 border-b border-black/50 border-x border-white/5 rounded-2xl p-8 shadow-xl shadow-black/60 backdrop-blur-sm">
                        <div class="space-y-6">
                            <div class="space-y-1.5">
                                <label
                                    class="block text-sm font-medium text-stone-400 ml-1"
                                    for="username"
                                >
                                    "Username"
                                </label>
                                <input
                                    class="w-full bg-[#232326] shadow-inner border border-white/5 rounded-lg text-white px-4 py-3 focus:outline-none focus:border-orange-500/50 focus:ring-1 focus:ring-orange-500/50 transition-all placeholder-stone-600"
                                    type="text"
                                    name="username"
                                    id="username"
                                    required
                                    minlength="3"
                                    autocomplete="username"
                                    placeholder="Username"
                                />
                            </div>

                            <div class="space-y-1.5">
                                <label
                                    class="block text-sm font-medium text-stone-400 ml-1"
                                    for="password"
                                >
                                    "Password"
                                </label>
                                <input
                                    class="w-full bg-[#232326] shadow-inner border border-white/5 rounded-lg text-white px-4 py-3 focus:outline-none focus:border-orange-500/50 focus:ring-1 focus:ring-orange-500/50 transition-all placeholder-stone-600"
                                    type="password"
                                    name="password"
                                    id="password"
                                    required
                                    minlength="4"
                                    autocomplete="current-password"
                                    placeholder="••••••••"
                                />
                            </div>

                            <button
                                class="w-full relative group overflow-hidden bg-gradient-to-b from-[#e35b2d] to-[#c2411c] hover:to-[#d64c23] border-t border-white/20 border-b border-black/20 text-white font-semibold py-3 rounded-lg transition-all duration-200 shadow-lg shadow-orange-900/40 active:scale-[0.98] active:shadow-none mt-4"
                                type="submit"
                                disabled=move || pending.get()
                            >
                                {move || if pending.get() { "Signing in..." } else { "Sign In" }}
                            </button>
                        </div>
                    </div>
                </ActionForm>

                <Show when=has_success fallback=|| ()>
                    <div class="bg-green-900/20 border border-green-500/20 text-green-200 px-4 py-3 rounded-lg text-center text-sm shadow-lg">
                        <span>"Login successful."</span>
                    </div>
                </Show>

                <Show when=move || error_message().is_some() fallback=|| ()>
                    <div class="bg-red-900/20 border border-red-500/20 text-red-200 px-4 py-3 rounded-lg text-center text-sm shadow-lg">
                        <span>{move || error_message().unwrap_or_default()}</span>
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[server]
pub async fn login(username: String, password: String) -> Result<(), AppError> {
    use crate::webui::authenticate_user;
    use crate::webui::cookies::set_session_cookie;
    use axum_client_ip::ClientIp;
    use leptos_axum::ResponseOptions;

    // Extract client IP and response context
    let response = expect_context::<ResponseOptions>();
    let ClientIp(client_ip) = leptos_axum::extract().await?;

    // Authenticate user and issue session
    let session = authenticate_user(&username, &password, client_ip).await?;

    // Set the session cookie
    set_session_cookie(&response, session.id.as_str());

    // note: server-side redirect doesn't work with streaming SSR.
    // client-side redirect is handled in the [LoginScreen] component via an Effect.
    Ok(())
}
