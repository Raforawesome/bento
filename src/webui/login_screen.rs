use crate::webui::AppError;
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

    // Handle client-side redirect after successful login with full page reload
    Effect::watch(
        move || action_value.get(),
        move |result, _, _| {
            if matches!(result.as_ref(), Some(Ok(_))) {
                // Force a full page reload to ensure session is properly loaded
                let _ = window().location().set_href("/");
            }
        },
        false,
    );

    view! {
        <div class="min-h-screen bg-base-200 flex items-center justify-center px-4 py-10">
            <div class="w-full max-w-md space-y-6">
                <div class="text-center space-y-2">
                    <img class="mx-auto" src="/bento-dark-64.webp" width=64 height=64 alt="Bento logo" />
                    <h1 class="text-3xl font-semibold">"Sign in to Bento"</h1>
                    <p class="text-base-content/70">"Enter your username and password to continue."</p>
                </div>

                <ActionForm action=login_action>
                    <div class="card bg-base-100 shadow-xl">
                        <div class="card-body space-y-4">
                            <label class="form-control w-full">
                                <span class="label-text font-medium px-1">"Username"</span>
                                <input
                                    class="input input-bordered w-full"
                                    type="text"
                                    name="username"
                                    required
                                    minlength="3"
                                    autocomplete="username"
                                    placeholder="Username"
                                />
                            </label>

                            <label class="form-control w-full">
                                <span class="label-text font-medium px-1">"Password"</span>
                                <input
                                    class="input input-bordered w-full"
                                    type="password"
                                    name="password"
                                    required
                                    minlength="4"
                                    autocomplete="current-password"
                                    placeholder="••••••••"
                                />
                            </label>

                            <button class="btn btn-primary w-full" type="submit" disabled=move || pending.get()>
                                {move || if pending.get() { "Signing in..." } else { "Sign In" }}
                            </button>
                        </div>
                    </div>
                </ActionForm>

                <Show when=has_success fallback=|| ()>
                    <div class="alert alert-success">
                        <span>"Login successful."</span>
                    </div>
                </Show>

                <Show
                    when=move || error_message().is_some()
                    fallback=|| ()
                >
                    <div class="alert alert-error">
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
    use axum::http::header::{HeaderValue, SET_COOKIE};
    use axum_client_ip::ClientIp;
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use leptos_axum::ResponseOptions;

    // Extract client IP and response context
    let response = expect_context::<ResponseOptions>();
    let ClientIp(client_ip) = leptos_axum::extract().await?;

    // Authenticate user and issue session
    let session = authenticate_user(&username, &password, client_ip).await?;

    // Set the session cookie
    let cookie = Cookie::build(("session_id", session.id.as_str().to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    #[cfg(not(debug_assertions))]
    let cookie = cookie.secure(true); // make cookie secure in release builds

    let cookie = cookie.build();

    if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
        response.insert_header(SET_COOKIE, header_value);
    }

    // note: server-side redirect doesn't work with streaming SSR.
    // client-side redirect is handled in the [LoginScreen] component via an Effect.
    Ok(())
}
