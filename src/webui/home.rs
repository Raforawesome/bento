use crate::webui::{AppError, CurrentUser, LogoSvg, get_current_user};
use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    // Create the user resource once at the top level
    let user = Resource::new(|| (), |_| get_current_user());

    view! {
        <DashboardPage user/>
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
pub fn DashboardPage(user: Resource<Result<Option<CurrentUser>, AppError>>) -> impl IntoView {
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

    let user_name = move || {
        user.get()
            .map(|res| match res {
                Ok(Some(user_info)) => user_info.username,
                _ => "Guest".to_string(),
            })
            .unwrap_or("Guest".to_string())
    };

    view! {
        <div class="min-h-screen bg-[#13141c] text-white font-sans selection:bg-orange-500/30">
            <NavBar user/>

            <main class="max-w-7xl mx-auto px-6 py-10">
                // header section
                <div class="mb-10">
                    <h1 class="text-3xl font-bold mb-2 tracking-tight">"Your Projects"</h1>
                    <p class="text-gray-400">
                        {format!("Welcome back, {}. Manage your projects or create a new one.", user_name())}
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
fn NavBar(user: Resource<Result<Option<CurrentUser>, AppError>>) -> impl IntoView {
    view! {
        <nav class="flex items-center justify-between px-6 py-4 border-b border-gray-800/60 bg-[#16171f]">
            // Left side: Logo
            <div class="flex items-center space-x-3">
                // <img src="/bento-dark-64.webp" alt="Bento Logo" class="w-8 h-8 opacity-90" />
                <LogoSvg size=8 />
                <span class="text-xl font-bold text-white tracking-tight">"Bento"</span>
            </div>

            // Right side: Icons and User Pill
            <div class="flex items-center space-x-5 text-gray-400">
                <button class="hover:text-white transition p-1">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                    </svg>
                </button>
                <button class="hover:text-white transition p-1 mr-2">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M14.857 17.082a23.848 23.848 0 0 0 5.454-1.31A8.967 8.967 0 0 1 18 9.75V9A6 6 0 0 0 6 9v.75a8.967 8.967 0 0 1-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 0 1-5.714 0m5.714 0a3 3 0 1 1-5.714 0" />
                    </svg>
                </button>

                // User Dropdown - THE PILL
                // Wrapped in a rounded-full border container with background
                <div class="flex items-center space-x-3 cursor-pointer transition bg-[#1f2029] hover:bg-[#252630] border border-gray-700/50 rounded-full py-1.5 pl-1.5 pr-4">
                    // fake avatar circle for now (decide on avatar feature later)
                    <div class="w-7 h-7 bg-gradient-to-br from-gray-600 to-gray-700 rounded-full flex items-center justify-center text-xs text-white font-bold border border-gray-600">
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-4 h-4 text-gray-300">
                          <path fill-rule="evenodd" d="M10 9a3 3 0 1 0 0-6 3 3 0 0 0 0 6Zm-7 9a7 7 0 1 1 14 0H3Z" clip-rule="evenodd" />
                        </svg>
                    </div>

                    {move || {
                        user.get().map(|result| {
                            match result {
                                Ok(Some(user_info)) => view! {
                                    <span class="text-sm font-medium text-gray-200">{user_info.username}</span>
                                }.into_any(),
                                _ => view! {
                                    <span class="text-sm font-medium text-gray-200">"Guest"</span>
                                }.into_any(),
                            }
                        })
                    }}
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-3 h-3 text-gray-500">
                      <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
                    </svg>
                </div>
            </div>
        </nav>
    }
}

#[component]
fn NewProjectCard() -> impl IntoView {
    view! {
        <div class="group h-full min-h-[280px] rounded-2xl border-2 border-dashed border-gray-700/40 bg-[#16171e] hover:bg-[#1a1b23] hover:border-orange-500/40 p-6 flex flex-col items-center justify-center text-center transition-all duration-300 relative overflow-hidden">

            // Subtle background glow effect on hover
            <div class="absolute inset-0 bg-gradient-to-tr from-orange-500/0 via-orange-500/0 to-orange-500/0 group-hover:to-orange-500/5 transition-all duration-500"></div>

            <div class="relative z-10 w-14 h-14 bg-[#252630] rounded-full flex items-center justify-center mb-5 text-orange-500 group-hover:scale-110 transition-transform duration-300 shadow-lg shadow-black/20">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-6 h-6">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                </svg>
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
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=icon_class>
                          <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 6.375c0 2.278-3.694 4.125-8.25 4.125S3.75 8.653 3.75 6.375m16.5 0c0-2.278-3.694-4.125-8.25-4.125S3.75 4.097 3.75 6.375m16.5 0v11.25c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125V6.375m16.5 0v3.75m-16.5-3.75v3.75m16.5 0v3.75C20.25 16.153 16.556 18 12 18s-8.25-1.847-8.25-4.125v-3.75m16.5 0c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125" />
                        </svg>
                        <span class="text-gray-400 text-sm"><strong class="text-gray-200 font-medium mr-1.5">{data.db_used}</strong> "Storage"</span>
                    </div>

                    <div class="flex items-center">
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=icon_class>
                           <path stroke-linecap="round" stroke-linejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 0 0 2.25-2.25v-6.75a2.25 2.25 0 0 0-2.25-2.25H6.75a2.25 2.25 0 0 0-2.25 2.25v6.75a2.25 2.25 0 0 0 2.25 2.25Z" />
                         </svg>
                         <span class="text-gray-400 text-sm"><strong class="text-gray-200 font-medium mr-1.5">{data.users_count}</strong> "Users"</span>
                    </div>

                    <div class="flex items-center">
                          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=icon_class>
                           <path stroke-linecap="round" stroke-linejoin="round" d="M7.5 14.25v2.25m3-4.5v4.5m3-6.75v6.75m3-9v9M6 20.25h12A2.25 2.25 0 0 0 20.25 18V6A2.25 2.25 0 0 0 18 3.75H6A2.25 2.25 0 0 0 3.75 6v12A2.25 2.25 0 0 0 6 20.25Z" />
                         </svg>
                         <span class="text-gray-400 text-sm"><strong class="text-gray-200 font-medium mr-1.5">{data.active_connections}</strong> "Connections"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
