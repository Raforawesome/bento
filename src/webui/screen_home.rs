use crate::types::AppError;
use crate::webui::icons::*;
use crate::webui::{CurrentUser, LogoSvg};
use leptos::form::ActionForm;
use leptos::prelude::*;

#[component]
pub fn Home(user: CurrentUser) -> impl IntoView {
    view! {
        <DashboardPage user=user.clone()/>
    }
}

#[derive(Clone, PartialEq)]
pub struct ProjectData {
    pub id: usize, // unique ID for iteration keys
    pub name: String,
    pub project_id: String,
    pub db_used: String,
    pub users_count: String,
    pub active_connections: String,
}

#[component]
pub fn DashboardPage(user: CurrentUser) -> impl IntoView {
    // mock data
    let projects = vec![
        ProjectData {
            id: 1,
            name: "My Awesome Project".to_string(),
            project_id: "proj_a1b2c3d4".to_string(),
            db_used: "12.5GB".to_string(),
            users_count: "1,204".to_string(),
            active_connections: "128".to_string(),
        },
        ProjectData {
            id: 2,
            name: "E-commerce API".to_string(),
            project_id: "proj_e5f6g7h8".to_string(),
            db_used: "38.2GB".to_string(),
            users_count: "25,890".to_string(),
            active_connections: "1,502".to_string(),
        },
        ProjectData {
            id: 3,
            name: "Mobile Game Backend".to_string(),
            project_id: "proj_i9j0k1l2".to_string(),
            db_used: "45.1GB".to_string(),
            users_count: "150,432".to_string(),
            active_connections: "8,912".to_string(),
        },
    ];

    let user_name = user.username.clone();

    view! {
        <div class="min-h-screen bg-[#13141c] text-white font-sans selection:bg-orange-500/30">
            <NavBar user=user.clone()/>

            <main class="max-w-7xl mx-auto px-6 py-10">
                // header section
                <div class="mb-10">
                    <h1 class="text-3xl font-bold mb-2 tracking-tight">"Your Projects"</h1>
                    <p class="text-gray-400">
                        {format!("Welcome back, {}. Manage your projects or create a new one.", user_name)}
                    </p>
                </div>

                // Grid Layout
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                    <NewProjectCard/>

                    {projects.into_iter().map(|project| {
                        view! { <ProjectCard data=project/> }
                    }).collect_view()}
                </div>
            </main>
        </div>
    }
}

#[component]
fn NavBar(user: CurrentUser) -> impl IntoView {
    let logout_action = ServerAction::<LogoutAction>::new();
    let pending = logout_action.pending();

    // Dropdown open/closed state
    let (dropdown_open, set_dropdown_open) = signal(false);

    // Handle redirect after successful logout
    Effect::watch(
        move || logout_action.value().get(),
        move |result, _, _| {
            if matches!(result.as_ref(), Some(Ok(_))) {
                let _ = window().location().set_href("/");
            }
        },
        false,
    );

    let username = user.username.clone();

    view! {
        <nav class="flex items-center justify-between px-6 py-4 border-b border-gray-800/60 bg-[#16171f]">
            // Left side: Logo
            <div class="flex items-center space-x-3">
                // <img src="/bento-dark-64.webp" alt="Bento Logo" class="w-8 h-8 opacity-90" />
                <LogoSvg size=8 />
                <span class="text-xl font-bold text-white tracking-tight">"Bento"</span>
            </div>

            // Right side: Icons and User Info
            <div class="flex items-center space-x-5 text-gray-400">
                <button class="hover:text-white transition p-1">
                    <DocumentIcon class="w-5 h-5" />
                </button>
                <button class="hover:text-white transition p-1">
                    <BellIcon class="w-5 h-5" />
                </button>

                // User Pill Dropdown
                <div class="relative">
                    // Dropdown trigger (User Pill)
                    <button
                        on:click=move |_| set_dropdown_open.update(|open| *open = !*open)
                        class="flex items-center space-x-3 bg-[#1f2029] border border-gray-700/50 rounded-full py-1.5 pl-1.5 pr-4 hover:bg-[#252630] hover:border-gray-600 transition cursor-pointer"
                    >
                        // fake avatar circle for now (decide on avatar feature later)
                        <div class="w-7 h-7 bg-gradient-to-br from-gray-600 to-gray-700 rounded-full flex items-center justify-center text-xs text-white font-bold border border-gray-600">
                            <UserIcon class="w-4 h-4 text-gray-300" />
                        </div>

                        <span class="text-sm font-medium text-gray-200">
                            {username}
                        </span>

                        // Chevron indicator
                        <svg
                            class="w-4 h-4 text-gray-400 transition-transform"
                            class:rotate-180=move || dropdown_open.get()
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                    </button>

                    // Dropdown Menu
                    <Show when=move || dropdown_open.get()>
                        // Backdrop to close dropdown when clicking outside
                        <div
                            class="fixed inset-0 z-10"
                            on:click=move |_| set_dropdown_open.set(false)
                        />

                        // Dropdown content
                        <div class="absolute right-0 mt-2 w-48 bg-[#1f2029] border border-gray-700/50 rounded-xl shadow-xl shadow-black/30 z-20 overflow-hidden">
                            // Manage users option
                            <button
                                class="flex items-center w-full px-4 py-3 text-sm text-gray-300 hover:bg-[#252630] hover:text-white transition"
                                on:click=move |_| set_dropdown_open.set(false)
                            >
                                <svg class="w-4 h-4 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
                                </svg>
                                "Manage Users"
                            </button>

                            // Divider
                            <div class="border-t border-gray-700/50" />

                            // Logout option
                            <ActionForm action=logout_action>
                                <button
                                    type="submit"
                                    class="flex items-center w-full px-4 py-3 text-sm text-gray-300 hover:bg-[#252630] hover:text-white transition"
                                    disabled=move || pending.get()
                                >
                                    <LogoutIcon class="w-4 h-4 mr-3" />
                                    <span>{move || if pending.get() { "Logging out..." } else { "Logout" }}</span>
                                </button>
                            </ActionForm>
                        </div>
                    </Show>
                </div>
            </div>
        </nav>
    }
}

/// Server function for logout - generates LogoutAction type for ActionForm
#[server]
pub async fn logout_action() -> Result<(), AppError> {
    crate::webui::logout().await
}

#[component]
fn NewProjectCard() -> impl IntoView {
    view! {
        <div class="group h-full min-h-[280px] rounded-2xl border-2 border-dashed border-gray-700/40 bg-[#16171e] hover:bg-[#1a1b23] hover:border-orange-500/40 p-6 flex flex-col items-center justify-center text-center transition-all duration-300 relative overflow-hidden">

            // Subtle background glow effect on hover
            <div class="absolute inset-0 bg-gradient-to-tr from-orange-500/0 via-orange-500/0 to-orange-500/0 group-hover:to-orange-500/5 transition-all duration-500"></div>

            <div class="relative z-10 w-14 h-14 bg-[#252630] rounded-full flex items-center justify-center mb-5 text-orange-500 group-hover:scale-110 transition-transform duration-300 shadow-lg shadow-black/20">
                <PlusIcon class="w-6 h-6" />
            </div>

            <h3 class="relative z-10 text-lg font-semibold mb-2 text-gray-200">"New Project"</h3>
            <p class="relative z-10 text-gray-500 text-sm mb-8">"Set up a new backend in seconds."</p>

            // Button with explicit glow
            <button class="relative z-10 bg-[#e35b2d] hover:bg-[#ff6b3d] text-white text-sm font-semibold py-2.5 px-6 rounded-lg w-full transition-all duration-300 shadow-lg shadow-orange-900/30 hover:shadow-orange-600/40 transform hover:-translate-y-0.5">
                "Create Project"
            </button>
        </div>
    }
}

#[component]
fn ProjectCard(data: ProjectData) -> impl IntoView {
    let icon_class = "w-4 h-4 text-gray-600 mr-2.5";

    view! {
        // Darker BG (#1e1f25), Border, and Shadow-XL to mimic the "RustBaaS" cards
        <div class="bg-[#1e1f25] border border-gray-800/60 rounded-2xl p-6 flex flex-col h-full justify-between shadow-xl shadow-black/20 hover:border-gray-700 transition-all duration-200">
            <div>
                // Card Header
                <div class="flex justify-between items-start mb-2">
                    <h3 class="text-[17px] font-semibold truncate pr-4 text-gray-100">{data.name}</h3>
                </div>

                // Styling the ID slightly smaller and darker
                <p class="text-gray-500 text-xs font-mono mb-8 flex items-center">
                    <span class="w-2 h-2 rounded-full bg-gray-700 mr-2"></span>
                    {data.project_id}
                </p>

                // Metrics List
                <div class="space-y-4">
                    <div class="flex items-center">
                        <DatabaseIcon class=icon_class />
                        <span class="text-gray-400 text-sm"><strong class="text-gray-200 font-medium mr-1.5">{data.db_used}</strong> "Storage"</span>
                    </div>

                    <div class="flex items-center">
                        <LockIcon class=icon_class />
                        <span class="text-gray-400 text-sm"><strong class="text-gray-200 font-medium mr-1.5">{data.users_count}</strong> "Users"</span>
                    </div>

                    <div class="flex items-center">
                        <ChartBarIcon class=icon_class />
                        <span class="text-gray-400 text-sm"><strong class="text-gray-200 font-medium mr-1.5">{data.active_connections}</strong> "Connections"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
