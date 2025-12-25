use gloo::storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlInputElement, MouseEvent};
use yew::prelude::*;

mod components;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tab {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub is_loading: bool,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::from("Home"),
            url: String::from("graphite://home"),
            favicon: None,
            is_loading: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Download {
    pub id: u32,
    pub filename: String,
    pub completed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SearchEngine {
    Yahoo,
    Google,
    Bing,
    DuckDuckGo,
    Brave,
}

impl SearchEngine {
    fn get_search_url(&self, query: &str) -> String {
        let encoded = js_sys::encode_uri_component(query);
        match self {
            SearchEngine::Yahoo => format!("https://search.yahoo.com/search?p={}", encoded),
            SearchEngine::Google => format!("https://www.google.com/search?q={}", encoded),
            SearchEngine::Bing => format!("https://www.bing.com/search?q={}", encoded),
            SearchEngine::DuckDuckGo => format!("https://duckduckgo.com/?q={}", encoded),
            SearchEngine::Brave => format!("https://search.brave.com/search?q={}", encoded),
        }
    }

    fn get_icon(&self) -> &'static str {
        match self {
            SearchEngine::Yahoo => "https://www.yahoo.com/favicon.ico",
            SearchEngine::Google => "https://www.google.com/favicon.ico",
            SearchEngine::Bing => "https://www.bing.com/favicon.ico",
            SearchEngine::DuckDuckGo => "https://duckduckgo.com/favicon.ico",
            SearchEngine::Brave => "https://brave.com/static-assets/images/brave-favicon.png",
        }
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        SearchEngine::Google
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BrowserState {
    pub tabs: Vec<Tab>,
    pub active_tab_id: u32,
    pub next_tab_id: u32,
    pub search_engine: SearchEngine,
    pub proxy_server: String,
    pub downloads: Vec<Download>,
    pub history: Vec<String>,
    pub history_index: usize,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::default()],
            active_tab_id: 0,
            next_tab_id: 1,
            search_engine: SearchEngine::default(),
            proxy_server: String::new(),
            downloads: vec![
                Download { id: 0, filename: "google.png".into(), completed: true },
                Download { id: 1, filename: "graphiteiscool.txt".into(), completed: true },
                Download { id: 2, filename: "vscode.exe".into(), completed: true },
            ],
            history: vec![],
            history_index: 0,
        }
    }
}

pub enum Msg {
    NewTab,
    CloseTab(u32),
    SelectTab(u32),
    Navigate(String),
    GoBack,
    GoForward,
    Reload,
    GoHome,
    UpdateUrlBar(String),
    SetSearchEngine(SearchEngine),
    SetProxyServer(String),
    ToggleSettingsPanel,
    ToggleDownloadsPanel,
    DeleteDownload(u32),
    OpenDownloadFolder(u32),
    DragStart(u32),
    DragOver(u32),
    DragEnd,
    CloseAllPanels,
    NoOp,
}

pub struct App {
    state: BrowserState,
    url_input: String,
    show_settings: bool,
    show_downloads: bool,
    dragging_tab: Option<u32>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let state = LocalStorage::get::<BrowserState>("graphite_state")
            .unwrap_or_default();
        
        // Don't show graphite://home in URL bar - show empty string
        let url_input = state.tabs
            .iter()
            .find(|t| t.id == state.active_tab_id)
            .map(|t| {
                if t.url == "graphite://home" {
                    String::new()
                } else {
                    t.url.clone()
                }
            })
            .unwrap_or_default();

        Self {
            state,
            url_input,
            show_settings: false,
            show_downloads: false,
            dragging_tab: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NewTab => {
                let new_tab = Tab {
                    id: self.state.next_tab_id,
                    ..Tab::default()
                };
                self.state.tabs.push(new_tab);
                self.state.active_tab_id = self.state.next_tab_id;
                self.state.next_tab_id += 1;
                self.url_input = String::new(); // Don't show graphite://home
                self.save_state();
                true
            }
            Msg::CloseTab(id) => {
                if self.state.tabs.len() > 1 {
                    let idx = self.state.tabs.iter().position(|t| t.id == id);
                    if let Some(idx) = idx {
                        self.state.tabs.remove(idx);
                        if self.state.active_tab_id == id {
                            let new_idx = idx.saturating_sub(1).min(self.state.tabs.len() - 1);
                            self.state.active_tab_id = self.state.tabs[new_idx].id;
                            let new_url = &self.state.tabs[new_idx].url;
                            self.url_input = if new_url == "graphite://home" {
                                String::new()
                            } else {
                                new_url.clone()
                            };
                        }
                    }
                    self.save_state();
                }
                true
            }
            Msg::SelectTab(id) => {
                self.state.active_tab_id = id;
                if let Some(tab) = self.state.tabs.iter().find(|t| t.id == id) {
                    self.url_input = if tab.url == "graphite://home" {
                        String::new()
                    } else {
                        tab.url.clone()
                    };
                }
                self.save_state();
                true
            }
            Msg::Navigate(url) => {
                let final_url = self.process_url(&url);
                let title = Self::get_title_from_url(&final_url);
                if let Some(tab) = self.state.tabs.iter_mut().find(|t| t.id == self.state.active_tab_id) {
                    tab.url = final_url.clone();
                    tab.title = title;
                    tab.is_loading = true;
                }
                self.url_input = final_url;
                self.save_state();
                true
            }
            Msg::GoBack => {
                // Go back in iframe history
                true
            }
            Msg::GoForward => {
                // Go forward in iframe history
                true
            }
            Msg::Reload => {
                if let Some(tab) = self.state.tabs.iter_mut().find(|t| t.id == self.state.active_tab_id) {
                    tab.is_loading = true;
                }
                true
            }
            Msg::GoHome => {
                if let Some(tab) = self.state.tabs.iter_mut().find(|t| t.id == self.state.active_tab_id) {
                    tab.url = String::from("graphite://home");
                    tab.title = String::from("Home");
                    tab.is_loading = false;
                }
                self.url_input = String::new(); // Don't show graphite://home
                self.save_state();
                true
            }
            Msg::UpdateUrlBar(value) => {
                self.url_input = value;
                true
            }
            Msg::SetSearchEngine(engine) => {
                self.state.search_engine = engine;
                self.save_state();
                true
            }
            Msg::SetProxyServer(proxy) => {
                self.state.proxy_server = proxy;
                self.save_state();
                true
            }
            Msg::ToggleSettingsPanel => {
                self.show_settings = !self.show_settings;
                self.show_downloads = false;
                true
            }
            Msg::ToggleDownloadsPanel => {
                self.show_downloads = !self.show_downloads;
                self.show_settings = false;
                true
            }
            Msg::DeleteDownload(id) => {
                self.state.downloads.retain(|d| d.id != id);
                self.save_state();
                true
            }
            Msg::OpenDownloadFolder(_id) => {
                // In WASM, we can't open file explorer
                true
            }
            Msg::DragStart(id) => {
                self.dragging_tab = Some(id);
                true
            }
            Msg::DragOver(target_id) => {
                if let Some(drag_id) = self.dragging_tab {
                    if drag_id != target_id {
                        let drag_idx = self.state.tabs.iter().position(|t| t.id == drag_id);
                        let target_idx = self.state.tabs.iter().position(|t| t.id == target_id);
                        if let (Some(from), Some(to)) = (drag_idx, target_idx) {
                            let tab = self.state.tabs.remove(from);
                            self.state.tabs.insert(to, tab);
                        }
                    }
                }
                true
            }
            Msg::DragEnd => {
                self.dragging_tab = None;
                self.save_state();
                true
            }
            Msg::CloseAllPanels => {
                self.show_settings = false;
                self.show_downloads = false;
                true
            }
            Msg::NoOp => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let active_tab = self.state.tabs.iter().find(|t| t.id == self.state.active_tab_id);
        let is_home = active_tab.map(|t| t.url == "graphite://home").unwrap_or(true);

        html! {
            <div class="browser-container" onclick={link.callback(|_| Msg::CloseAllPanels)}>
                // Tab Bar
                <div class="tab-bar">
                    { for self.state.tabs.iter().map(|tab| {
                        let is_active = tab.id == self.state.active_tab_id;
                        let tab_id = tab.id;
                        let close_id = tab.id;
                        let drag_id = tab.id;
                        let drop_id = tab.id;
                        
                        html! {
                            <div 
                                class={classes!("tab", is_active.then_some("active"))}
                                onclick={link.callback(move |_| Msg::SelectTab(tab_id))}
                                draggable="true"
                                ondragstart={link.callback(move |_| Msg::DragStart(drag_id))}
                                ondragover={link.callback(move |e: DragEvent| {
                                    e.prevent_default();
                                    Msg::DragOver(drop_id)
                                })}
                                ondragend={link.callback(|_| Msg::DragEnd)}
                            >
                                <span class="tab-favicon icon icon-home"></span>
                                <span class="tab-title">{&tab.title}</span>
                                <button 
                                    class="tab-close"
                                    onclick={link.callback(move |e: MouseEvent| {
                                        e.stop_propagation();
                                        Msg::CloseTab(close_id)
                                    })}
                                ><span class="icon icon-close"></span></button>
                            </div>
                        }
                    })}
                    <button class="new-tab-btn" onclick={link.callback(|_| Msg::NewTab)}><span class="icon icon-add"></span></button>
                </div>

                // Navigation Bar
                <div class="nav-bar">
                    <div class="nav-controls">
                        <button class="nav-btn" onclick={link.callback(|_| Msg::GoBack)} title="Back">
                            <span class="icon icon-arrow-back"></span>
                        </button>
                        <button class="nav-btn" onclick={link.callback(|_| Msg::GoForward)} title="Forward">
                            <span class="icon icon-arrow-forward"></span>
                        </button>
                        <button class="nav-btn" onclick={link.callback(|_| Msg::Reload)} title="Reload">
                            <span class="icon icon-refresh"></span>
                        </button>
                    </div>
                    
                    <div class="url-bar-container">
                        <input 
                            type="text" 
                            class="url-bar"
                            placeholder="Search or enter a URL..."
                            value={self.url_input.clone()}
                            oninput={link.callback(|e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                Msg::UpdateUrlBar(input.value())
                            })}
                            onkeypress={link.callback(|e: KeyboardEvent| {
                                if e.key() == "Enter" {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    Msg::Navigate(input.value())
                                } else {
                                    Msg::NoOp
                                }
                            })}
                        />
                        <button class="url-bar-search-btn" title="Search">
                            <span class="icon icon-search"></span>
                        </button>
                    </div>

                    <div class="toolbar-icons">
                        <button class="toolbar-btn" title="Toggle Dark Mode">
                            <span class="icon icon-light-mode"></span>
                        </button>
                        <button class="toolbar-btn" onclick={link.callback(|_| Msg::GoHome)} title="Home">
                            <span class="icon icon-home"></span>
                        </button>
                        <button 
                            class="toolbar-btn" 
                            onclick={link.callback(|e: MouseEvent| {
                                e.stop_propagation();
                                Msg::ToggleDownloadsPanel
                            })}
                            title="Downloads"
                        >
                            <span class="icon icon-download"></span>
                        </button>
                        <button 
                            class="toolbar-btn" 
                            onclick={link.callback(|e: MouseEvent| {
                                e.stop_propagation();
                                Msg::ToggleSettingsPanel
                            })}
                            title="Settings"
                        >
                            <span class="icon icon-settings"></span>
                        </button>
                    </div>
                </div>

                // Content Area
                <div class="content-area">
                    if is_home {
                        <div class="home-page">
                            <h1 class="browser-title">{"graphite"}</h1>
                            <p class="browser-tagline">{"a simple, sleek, modern, minimalist web browser"}</p>
                            <div class="home-search-container">
                                <input 
                                    type="text" 
                                    class="home-search"
                                    placeholder="Search or enter a URL"
                                    onkeypress={link.callback(|e: KeyboardEvent| {
                                        if e.key() == "Enter" {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            Msg::Navigate(input.value())
                                        } else {
                                            Msg::NoOp
                                        }
                                    })}
                                />
                                <button class="home-search-btn">
                                    <span class="icon icon-search"></span>
                                </button>
                            </div>
                        </div>
                    } else {
                        <iframe 
                            class="browser-iframe"
                            src={self.get_proxied_url(active_tab.map(|t| &t.url).unwrap_or(&String::new()))}
                            sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
                        />
                    }
                </div>

                // Settings Panel
                if self.show_settings {
                    <div class="panel settings-panel" onclick={|e: MouseEvent| e.stop_propagation()}>
                        <div class="panel-header">
                            <span class="panel-icon icon icon-search"></span>
                            <span class="panel-title">{"Search Engine"}</span>
                        </div>
                        <div class="search-engines">
                            { self.render_search_engine_option(link, SearchEngine::Yahoo, "Y!", "#6001d2") }
                            { self.render_search_engine_option(link, SearchEngine::Google, "G", "#4285f4") }
                            { self.render_search_engine_option(link, SearchEngine::Bing, "b", "#00809d") }
                            { self.render_search_engine_option(link, SearchEngine::DuckDuckGo, "🦆", "#de5833") }
                            { self.render_search_engine_option(link, SearchEngine::Brave, "🦁", "#fb542b") }
                        </div>
                        <div class="panel-header proxy-header">
                            <span class="panel-icon icon icon-cell-tower"></span>
                            <span class="panel-title">{"Proxy Server"}</span>
                        </div>
                        <input 
                            type="text" 
                            class="proxy-input"
                            placeholder="Enter a wss:// or ws:// proxy"
                            value={self.state.proxy_server.clone()}
                            oninput={link.callback(|e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                Msg::SetProxyServer(input.value())
                            })}
                        />
                    </div>
                }

                // Downloads Panel
                if self.show_downloads {
                    <div class="panel downloads-panel" onclick={|e: MouseEvent| e.stop_propagation()}>
                        <div class="panel-header">
                            <span class="panel-icon icon icon-download"></span>
                            <span class="panel-title">{"Downloads"}</span>
                        </div>
                        <div class="downloads-list">
                            { for self.state.downloads.iter().map(|download| {
                                let dl_id = download.id;
                                let dl_id2 = download.id;
                                html! {
                                    <div class="download-item">
                                        <span class="download-name">{&download.filename}</span>
                                        <div class="download-actions">
                                            <button 
                                                class="download-btn"
                                                onclick={link.callback(move |_| Msg::OpenDownloadFolder(dl_id))}
                                                title="Open Folder"
                                            ><span class="icon icon-folder"></span></button>
                                            <button 
                                                class="download-btn"
                                                onclick={link.callback(move |_| Msg::DeleteDownload(dl_id2))}
                                                title="Delete"
                                            ><span class="icon icon-delete"></span></button>
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    </div>
                }
            </div>
        }
    }
}

impl App {
    fn save_state(&self) {
        let _ = LocalStorage::set("graphite_state", &self.state);
    }

    fn process_url(&self, input: &str) -> String {
        let input = input.trim();
        
        // Check if it's already a URL
        if input.starts_with("http://") || input.starts_with("https://") || input.starts_with("graphite://") {
            return input.to_string();
        }
        
        // Check if it looks like a domain
        if input.contains('.') && !input.contains(' ') {
            return format!("https://{}", input);
        }
        
        // Otherwise, treat as a search query
        self.state.search_engine.get_search_url(input)
    }

    fn get_proxied_url(&self, url: &str) -> String {
        if self.state.proxy_server.is_empty() {
            url.to_string()
        } else {
            // Use proxy server if configured
            format!("{}?url={}", self.state.proxy_server, js_sys::encode_uri_component(url))
        }
    }

    fn get_title_from_url(url: &str) -> String {
        if url.starts_with("graphite://") {
            return "Home".to_string();
        }
        
        // Extract domain from URL
        url.replace("https://", "")
            .replace("http://", "")
            .split('/')
            .next()
            .unwrap_or("New Tab")
            .to_string()
    }

    fn render_search_engine_option(&self, link: &yew::html::Scope<Self>, engine: SearchEngine, icon: &str, color: &str) -> Html {
        let is_selected = self.state.search_engine == engine;
        let engine_clone = engine.clone();
        
        html! {
            <button 
                class={classes!("search-engine-btn", is_selected.then_some("selected"))}
                style={format!("background-color: {};", color)}
                onclick={link.callback(move |_| Msg::SetSearchEngine(engine_clone.clone()))}
            >
                {icon}
            </button>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
