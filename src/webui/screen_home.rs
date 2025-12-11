use crate::types::{AppError, ProjectSummary};
use crate::webui::icons::*;
use crate::webui::{CurrentUser, LogoSvg, Logout, create_project, delete_project, get_my_projects};
use leptos::prelude::*;

type CreateProjectInput = (String, Option<String>);
type CreateProjectOutput = Result<ProjectSummary, AppError>;
type CreateProjectAction = Action<CreateProjectInput, CreateProjectOutput>;

type DeleteProjectOutput = Result<(), AppError>;
type DeleteProjectAction = Action<String, DeleteProjectOutput>;

// Context type to avoid prop drilling
#[derive(Clone)]
struct HomeContext {
    user: CurrentUser,
    projects_resource: Resource<Result<Vec<ProjectSummary>, AppError>>,
    create_action: CreateProjectAction,
    delete_action: DeleteProjectAction,
}

#[component]
pub fn HomeScreen(user: CurrentUser) -> impl IntoView {
    // Resource to fetch projects from the server
    let projects_resource = Resource::new(|| (), |_| get_my_projects());

    // Action to create a new project
    let create_action = Action::new(|(name, description): &CreateProjectInput| {
        let name = name.clone();
        let description = description.clone();
        async move { create_project(name, description).await }
    });

    // Action to delete a project
    let delete_action = Action::new(|project_id: &String| {
        let project_id = project_id.clone();
        async move { delete_project(project_id).await }
    });

    // Refetch projects when create or delete action completes successfully
    Effect::watch(
        move || create_action.value().get(),
        move |result, _, _| {
            if matches!(result.as_ref(), Some(Ok(_))) {
                projects_resource.refetch();
            }
        },
        false,
    );

    Effect::watch(
        move || delete_action.value().get(),
        move |result, _, _| {
            if matches!(result.as_ref(), Some(Ok(_))) {
                projects_resource.refetch();
            }
        },
        false,
    );

    let user_name = user.username.clone();

    // Provide context to child components
    let context = HomeContext {
        user: user.clone(),
        projects_resource,
        create_action,
        delete_action,
    };
    provide_context(context);

    view! {
        <div class="min-h-screen bg-[#13141c] text-white font-sans selection:bg-orange-500/30">
            <NavBar />

            <main class="max-w-7xl mx-auto px-6 py-10">
                // header section
                <div class="mb-10">
                    <h1 class="text-3xl font-bold mb-2 tracking-tight">"Your Projects"</h1>
                    <p class="text-gray-400">
                        {format!("Welcome back, {}. Manage your projects or create a new one.", user_name)}
                    </p>
                </div>

                // Grid Layout
                <Suspense fallback=ProjectsPlaceholder>
                    {move || {
                        projects_resource.get().map(|result| {
                            match result {
                                Ok(projects) => view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                                        <NewProjectCard />
                                        {projects.into_iter().map(|project| {
                                            view! { <ProjectCard project=project /> }
                                        }).collect_view()}
                                    </div>
                                }.into_any(),

                                Err(e) => view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                                        <NewProjectCard />
                                        <div class="col-span-3 bg-red-900/20 border border-red-800 rounded-2xl p-6 text-red-400">
                                            <p class="font-medium">"Failed to load projects"</p>
                                            <p class="text-sm mt-1">{e.to_string()}</p>
                                        </div>
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </main>
        </div>
    }
}

#[component]
fn ProjectsPlaceholder() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <NewProjectCard />
            // Loading skeleton cards
            <ProjectCardSkeleton />
            <ProjectCardSkeleton />
            <ProjectCardSkeleton />
        </div>
    }
}

#[component]
fn NavBar() -> impl IntoView {
    let logout_action = ServerAction::<Logout>::new();
    let pending = logout_action.pending();

    // Get context
    let context = expect_context::<HomeContext>();

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

    let username = context.user.username.clone();

    view! {
        <nav class="flex items-center justify-between px-6 py-4 border-b border-gray-800/60 bg-[#16171f]">
            // Left side: Logo
            <div class="flex items-center space-x-3">
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
                                <UserIcon class="w-4 h-4 mr-3" />
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

#[component]
fn NewProjectCard() -> impl IntoView {
    // Get context
    let context = expect_context::<HomeContext>();
    let create_action = context.create_action;

    let (show_form, set_show_form) = signal(false);
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());

    let pending = create_action.pending();

    // Close form and reset inputs after successful creation
    Effect::watch(
        move || create_action.value().get(),
        move |result, _, _| {
            if matches!(result.as_ref(), Some(Ok(_))) {
                set_show_form.set(false);
                set_name.set(String::new());
                set_description.set(String::new());
            }
        },
        false,
    );

    view! {
        <div class="group h-full min-h-[280px] rounded-2xl border-2 border-dashed border-gray-700/40 bg-[#16171e] hover:bg-[#1a1b23] hover:border-orange-500/40 p-6 flex flex-col items-center justify-center text-center transition-all duration-300 relative overflow-hidden">

            // Subtle background glow effect on hover
            <div class="absolute inset-0 bg-gradient-to-tr from-orange-500/0 via-orange-500/0 to-orange-500/0 group-hover:to-orange-500/5 transition-all duration-500"></div>

            <Show
                when=move || show_form.get()
                fallback=move || view! {
                    <div class="relative z-10 w-14 h-14 bg-[#252630] rounded-full flex items-center justify-center mb-5 text-orange-500 group-hover:scale-110 transition-transform duration-300 shadow-lg shadow-black/20">
                        <PlusIcon class="w-6 h-6" />
                    </div>

                    <h3 class="relative z-10 text-lg font-semibold mb-2 text-gray-200">"New Project"</h3>
                    <p class="relative z-10 text-gray-500 text-sm mb-8">"Set up a new backend in seconds."</p>

                    <button
                        class="relative z-10 bg-[#e35b2d] hover:bg-[#ff6b3d] text-white text-sm font-semibold py-2.5 px-6 rounded-lg w-full transition-all duration-300 shadow-lg shadow-orange-900/30 hover:shadow-orange-600/40 transform hover:-translate-y-0.5"
                        on:click=move |_| set_show_form.set(true)
                    >
                        "Create Project"
                    </button>
                }
            >
                // Create project form
                <form
                    class="relative z-10 w-full space-y-4"
                    on:submit=move |ev| {
                        ev.prevent_default();
                        let name_val = name.get();
                        let desc_val = description.get();
                        let desc = if desc_val.trim().is_empty() { None } else { Some(desc_val) };
                        create_action.dispatch((name_val, desc));
                    }
                >
                    <div class="text-left">
                        <label class="block text-sm font-medium text-gray-300 mb-1">"Project Name"</label>
                        <input
                            type="text"
                            required
                            class="w-full bg-[#252630] border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-orange-500 transition"
                            placeholder="My Awesome Project"
                            prop:value=move || name.get()
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="text-left">
                        <label class="block text-sm font-medium text-gray-300 mb-1">"Description"</label>
                        <textarea
                            class="w-full bg-[#252630] border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-orange-500 transition resize-none"
                            placeholder="Optional description..."
                            rows="2"
                            prop:value=move || description.get()
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="flex gap-2">
                        <button
                            type="button"
                            class="flex-1 bg-gray-700 hover:bg-gray-600 text-white text-sm font-medium py-2 px-4 rounded-lg transition"
                            on:click=move |_| {
                                set_show_form.set(false);
                                set_name.set(String::new());
                                set_description.set(String::new());
                            }
                            disabled=move || pending.get()
                        >
                            "Cancel"
                        </button>
                        <button
                            type="submit"
                            class="flex-1 bg-[#e35b2d] hover:bg-[#ff6b3d] text-white text-sm font-semibold py-2 px-4 rounded-lg transition disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || pending.get() || name.get().trim().is_empty()
                        >
                            {move || if pending.get() { "Creating..." } else { "Create" }}
                        </button>
                    </div>
                </form>
            </Show>
        </div>
    }
}

#[component]
fn ProjectCard(project: ProjectSummary) -> impl IntoView {
    // Get context
    let context = expect_context::<HomeContext>();
    let delete_action = context.delete_action;

    let icon_class = "w-4 h-4 text-gray-600 mr-2.5";
    let project_id = project.id.0.to_string();
    let project_id_for_delete = project_id.clone();

    let (show_delete_confirm, set_show_delete_confirm) = signal(false);
    let pending = delete_action.pending();

    // Close delete confirmation modal after successful delete
    Effect::watch(
        move || delete_action.value().get(),
        move |result, _, _| {
            if matches!(result.as_ref(), Some(Ok(_))) {
                set_show_delete_confirm.set(false);
            }
        },
        false,
    );

    // Format the created_at date
    let created_at = project.created_at;
    let date_str = format!(
        "{:04}-{:02}-{:02}",
        created_at.year(),
        created_at.month() as u8,
        created_at.day()
    );

    view! {
        <div class="bg-[#1e1f25] border border-gray-800/60 rounded-2xl p-6 flex flex-col h-full justify-between shadow-xl shadow-black/20 hover:border-gray-700 transition-all duration-200 relative group">
            // Delete button (shown on hover)
            <button
                class="absolute top-3 right-3 w-8 h-8 rounded-lg bg-red-900/0 hover:bg-red-900/50 flex items-center justify-center text-gray-500 hover:text-red-400 transition opacity-0 group-hover:opacity-100"
                on:click=move |_| set_show_delete_confirm.set(true)
            >
                <TrashIcon class="w-4 h-4" />
            </button>

            <div>
                // Card Header
                <div class="flex justify-between items-start mb-2">
                    <h3 class="text-[17px] font-semibold truncate pr-10 text-gray-100">{project.name.clone()}</h3>
                </div>

                // Project ID
                <p class="text-gray-500 text-xs font-mono mb-4 flex items-center">
                    <span class="w-2 h-2 rounded-full bg-green-500 mr-2"></span>
                    {project_id}
                </p>

                // Description if present
                {project.description.map(|desc| view! {
                    <p class="text-gray-400 text-sm mb-4 line-clamp-2">{desc}</p>
                })}

                // Metrics
                <div class="space-y-3">
                    <div class="flex items-center">
                        <CalendarIcon class=icon_class />
                        <span class="text-gray-400 text-sm">"Created: "<strong class="text-gray-200 font-medium">{date_str}</strong></span>
                    </div>

                    <div class="flex items-center">
                        <DatabaseIcon class=icon_class />
                        <span class="text-gray-400 text-sm"><strong class="text-gray-200 font-medium mr-1">"—"</strong> "Storage"</span>
                    </div>

                    <div class="flex items-center">
                        <LockIcon class=icon_class />
                        <span class="text-gray-400 text-sm"><strong class="text-gray-200 font-medium mr-1">"—"</strong> "Users"</span>
                    </div>
                </div>
            </div>

            // Delete confirmation modal
            <Show when=move || show_delete_confirm.get()>
                <div class="absolute inset-0 bg-[#1e1f25]/95 rounded-2xl flex flex-col items-center justify-center p-6 z-10">
                    <TrashIcon class="w-8 h-8 text-red-400 mb-3" />
                    <p class="text-gray-200 text-sm font-medium mb-1">"Delete this project?"</p>
                    <p class="text-gray-500 text-xs mb-4 text-center">"This action cannot be undone."</p>

                    <div class="flex gap-2 w-full">
                        <button
                            class="flex-1 bg-gray-700 hover:bg-gray-600 text-white text-sm font-medium py-2 px-4 rounded-lg transition"
                            on:click=move |_| set_show_delete_confirm.set(false)
                            disabled=move || pending.get()
                        >
                            "Cancel"
                        </button>
                        <button
                            type="button"
                            class="flex-1 bg-red-600 hover:bg-red-500 text-white text-sm font-semibold py-2 px-4 rounded-lg transition disabled:opacity-50"
                            disabled=move || pending.get()
                            on:click={
                                let project_id = project_id_for_delete.clone();
                                move |_| {
                                    delete_action.dispatch(project_id.clone());
                                }
                            }
                        >
                            {move || if pending.get() { "Deleting..." } else { "Delete" }}
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ProjectCardSkeleton() -> impl IntoView {
    view! {
        <div class="bg-[#1e1f25] border border-gray-800/60 rounded-2xl p-6 flex flex-col h-full min-h-[280px] shadow-xl shadow-black/20 animate-pulse">
            <div>
                // Title skeleton
                <div class="h-5 bg-gray-700/50 rounded w-3/4 mb-4"></div>

                // ID skeleton
                <div class="h-3 bg-gray-700/30 rounded w-full mb-8"></div>

                // Metrics skeletons
                <div class="space-y-4">
                    <div class="h-4 bg-gray-700/30 rounded w-2/3"></div>
                    <div class="h-4 bg-gray-700/30 rounded w-1/2"></div>
                    <div class="h-4 bg-gray-700/30 rounded w-3/5"></div>
                </div>
            </div>
        </div>
    }
}
