use crate::webui::get_current_user;
use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        // <h1>Hello, world</h1>
        <DashboardPage/>
    }
}

#[derive(Clone, PartialEq)]
pub struct ProjectData {
    pub id: usize, // Unique ID for iteration keys
    pub name: String,
    pub project_id: String,
    pub db_used: String,
    pub users_count: String,
    pub active_connections: String,
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    // Mock data for the dashboard. In a real app, this would come from a Resource or Signal.
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

    view! {
        <div class="min-h-screen bg-[#1a1b23] text-white font-sans">
            <NavBar/>

            <main class="max-w-7xl mx-auto px-6 py-8">
                // Header Section
                <div class="mb-8">
                    <h1 class="text-3xl font-bold mb-2">"Your Projects"</h1>
                    <p class="text-gray-400">"Welcome back, Jane. Manage your projects or create a new one."</p>
                </div>

                // Grid Layout
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                    // The "New Project" action card
                    <NewProjectCard/>

                    // Iteratively render existing project cards
                    {projects.into_iter().map(|project| {
                        view! { <ProjectCard data=project/> }
                    }).collect_view()}
                </div>
            </main>
        </div>
    }
}

#[component]
fn NavBar() -> impl IntoView {
    // Fetch the current user's information
    let user = Resource::new(|| (), |_| get_current_user());

    view! {
        <nav class="flex items-center justify-between px-6 py-4 border-b border-gray-800 bg-[#1a1b23]">
            // Left side: Logo and Title
            <div class="flex items-center space-x-3">
                 // Assuming image_1.png is saved as /bento-logo.png in your public assets
                <img src="/bento-dark-64.webp" alt="Bento Logo" class="w-8 h-8" />
                <span class="text-xl font-bold text-orange-500">"Bento"</span>
            </div>

            // Right side: Icons and User Profile
            <div class="flex items-center space-x-6 text-gray-400">
                <button class="hover:text-white transition">
                    // Document Icon
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                    </svg>
                </button>
                <button class="hover:text-white transition">
                    // Bell Icon
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M14.857 17.082a23.848 23.848 0 0 0 5.454-1.31A8.967 8.967 0 0 1 18 9.75V9A6 6 0 0 0 6 9v.75a8.967 8.967 0 0 1-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 0 1-5.714 0m5.714 0a3 3 0 1 1-5.714 0" />
                    </svg>
                </button>

                // User Dropdown - displays username
                <div class="flex items-center space-x-2 cursor-pointer hover:text-white transition">
                    {move || {
                        user.get().map(|result| {
                            match result {
                                Ok(Some(user_info)) => view! {
                                    <span class="font-medium">{user_info.username}</span>
                                }.into_any(),
                                _ => view! {
                                    <span class="font-medium">"Guest"</span>
                                }.into_any(),
                            }
                        })
                    }}
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
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
        <div class="bg-[#1a1b23] border-2 border-dashed border-gray-700 rounded-xl p-6 flex flex-col items-center justify-center text-center h-full min-h-[280px]">
            <div class="w-12 h-12 bg-gray-800 rounded-full flex items-center justify-center mb-4 text-orange-500">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-6 h-6">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                </svg>
            </div>
            <h3 class="text-lg font-semibold mb-2">"New Project"</h3>
            <p class="text-gray-400 text-sm mb-6">"Set up a new backend in seconds."</p>
            <button class="bg-orange-500 hover:bg-orange-600 text-white font-medium py-2 px-6 rounded-md w-full transition-colors">
                "Create Project"
            </button>
        </div>
    }
}

#[component]
fn ProjectCard(data: ProjectData) -> impl IntoView {
    // Common icon styles
    let icon_class = "w-5 h-5 text-gray-500 mr-3";

    view! {
        <div class="bg-[#252630] rounded-xl p-6 flex flex-col h-full justify-between">
            <div>
                // Card Header
                <div class="flex justify-between items-start mb-2">
                    <h3 class="text-lg font-semibold truncate pr-4">{data.name}</h3>

                </div>
                <p class="text-gray-400 text-sm mb-6">"ID: " {data.project_id}</p>

                // Metrics List
                <div class="space-y-4">
                    // Database Metric
                    <div class="flex items-center">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=icon_class>
                          <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 6.375c0 2.278-3.694 4.125-8.25 4.125S3.75 8.653 3.75 6.375m16.5 0c0-2.278-3.694-4.125-8.25-4.125S3.75 4.097 3.75 6.375m16.5 0v11.25c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125V6.375m16.5 0v3.75m-16.5-3.75v3.75m16.5 0v3.75C20.25 16.153 16.556 18 12 18s-8.25-1.847-8.25-4.125v-3.75m16.5 0c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125" />
                        </svg>
                        <span class="text-gray-300 text-sm"><strong class="text-white font-semibold mr-1">{data.db_used}</strong> "Database Used"</span>
                    </div>
                    // Users Metric
                    <div class="flex items-center">
                         <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=icon_class>
                          <path stroke-linecap="round" stroke-linejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 0 0 2.25-2.25v-6.75a2.25 2.25 0 0 0-2.25-2.25H6.75a2.25 2.25 0 0 0-2.25 2.25v6.75a2.25 2.25 0 0 0 2.25 2.25Z" />
                        </svg>
                        <span class="text-gray-300 text-sm"><strong class="text-white font-semibold mr-1">{data.users_count}</strong> "Users"</span>
                    </div>
                    // Connections Metric
                    <div class="flex items-center">
                         <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=icon_class>
                          <path stroke-linecap="round" stroke-linejoin="round" d="M7.5 14.25v2.25m3-4.5v4.5m3-6.75v6.75m3-9v9M6 20.25h12A2.25 2.25 0 0 0 20.25 18V6A2.25 2.25 0 0 0 18 3.75H6A2.25 2.25 0 0 0 3.75 6v12A2.25 2.25 0 0 0 6 20.25Z" />
                        </svg>
                        <span class="text-gray-300 text-sm"><strong class="text-white font-semibold mr-1">{data.active_connections}</strong> "Active Connections"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
