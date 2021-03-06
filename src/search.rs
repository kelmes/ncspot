use std::cell::RefCell;
use std::sync::{Arc, Mutex, RwLock};

use crate::album::Album;
use crate::artist::Artist;
use crate::command::{Command, MoveMode};
use crate::commands::CommandResult;
use crate::events::EventManager;
use crate::library::Library;
use crate::playlist::Playlist;
use crate::queue::Queue;
use crate::spotify::{Spotify, URIType};
use crate::track::Track;
use crate::traits::{ListItem, ViewExt};
// use crate::ui::listview::{ListView, Pagination};
// use crate::ui::tabview::TabView;

pub struct SearchView {
    pub results_tracks: Arc<RwLock<Vec<Track>>>,
    results_albums: Arc<RwLock<Vec<Album>>>,
    results_artists: Arc<RwLock<Vec<Artist>>>,
    results_playlists: Arc<RwLock<Vec<Playlist>>>,
    edit_focused: bool,
    events: EventManager,
    spotify: Arc<Spotify>,
}

type SearchHandler<I> =
    Box<dyn Fn(&Arc<Spotify>, &Arc<RwLock<Vec<I>>>, &str, usize, bool) -> u32 + Send + Sync>;

pub const LIST_ID: &str = "search_list";
pub const EDIT_ID: &str = "search_edit";
impl SearchView {
    pub fn new(
        events: EventManager,
        spotify: Arc<Spotify>,
        queue: Arc<Queue>,
        library: Arc<Library>,
    ) -> SearchView {
        let results_tracks = Arc::new(RwLock::new(Vec::new()));
        let results_albums = Arc::new(RwLock::new(Vec::new()));
        let results_artists = Arc::new(RwLock::new(Vec::new()));
        let results_playlists = Arc::new(RwLock::new(Vec::new()));

        SearchView {
            results_tracks,
            results_albums,
            results_artists,
            results_playlists,
            edit_focused: true,
            events,
            spotify,
        }
    }

    pub fn clear(&mut self) {
        //self.edit
        //    .call_on(&Selector::Name(EDIT_ID), |v: &mut EditView| {
        //        v.set_content("");
        //    });
    }

    fn get_track(
        spotify: &Arc<Spotify>,
        tracks: &Arc<RwLock<Vec<Track>>>,
        query: &str,
        _offset: usize,
        _append: bool,
    ) -> u32 {
        if let Some(results) = spotify.track(&query) {
            let t = vec![(&results).into()];
            let mut r = tracks.write().unwrap();
            *r = t;
            return 1;
        }
        0
    }

    fn search_track(
        spotify: &Arc<Spotify>,
        tracks: &Arc<RwLock<Vec<Track>>>,
        query: &str,
        offset: usize,
        append: bool,
    ) -> u32 {
        if let Some(results) = spotify.search_track(&query, 50, offset as u32) {
            let mut t = results.tracks.items.iter().map(|ft| ft.into()).collect();
            let mut r = tracks.write().unwrap();

            if append {
                r.append(&mut t);
            } else {
                *r = t;
            }
            return results.tracks.total;
        }
        0
    }

    fn get_album(
        spotify: &Arc<Spotify>,
        albums: &Arc<RwLock<Vec<Album>>>,
        query: &str,
        _offset: usize,
        _append: bool,
    ) -> u32 {
        if let Some(results) = spotify.album(&query) {
            let a = vec![(&results).into()];
            let mut r = albums.write().unwrap();
            *r = a;
            return 1;
        }
        0
    }

    fn search_album(
        spotify: &Arc<Spotify>,
        albums: &Arc<RwLock<Vec<Album>>>,
        query: &str,
        offset: usize,
        append: bool,
    ) -> u32 {
        if let Some(results) = spotify.search_album(&query, 50, offset as u32) {
            let mut a = results.albums.items.iter().map(|sa| sa.into()).collect();
            let mut r = albums.write().unwrap();

            if append {
                r.append(&mut a);
            } else {
                *r = a;
            }
            return results.albums.total;
        }
        0
    }

    fn get_artist(
        spotify: &Arc<Spotify>,
        artists: &Arc<RwLock<Vec<Artist>>>,
        query: &str,
        _offset: usize,
        _append: bool,
    ) -> u32 {
        if let Some(results) = spotify.artist(&query) {
            let a = vec![(&results).into()];
            let mut r = artists.write().unwrap();
            *r = a;
            return 1;
        }
        0
    }

    fn search_artist(
        spotify: &Arc<Spotify>,
        artists: &Arc<RwLock<Vec<Artist>>>,
        query: &str,
        offset: usize,
        append: bool,
    ) -> u32 {
        if let Some(results) = spotify.search_artist(&query, 50, offset as u32) {
            let mut a = results.artists.items.iter().map(|fa| fa.into()).collect();
            let mut r = artists.write().unwrap();

            if append {
                r.append(&mut a);
            } else {
                *r = a;
            }
            return results.artists.total;
        }
        0
    }

    fn get_playlist(
        spotify: &Arc<Spotify>,
        playlists: &Arc<RwLock<Vec<Playlist>>>,
        query: &str,
        _offset: usize,
        _append: bool,
    ) -> u32 {
        if let Some(result) = spotify.playlist(&query).as_ref() {
            let pls = vec![result.into()];
            let mut r = playlists.write().unwrap();
            *r = pls;
            return 1;
        }
        0
    }

    fn search_playlist(
        spotify: &Arc<Spotify>,
        playlists: &Arc<RwLock<Vec<Playlist>>>,
        query: &str,
        offset: usize,
        append: bool,
    ) -> u32 {
        if let Some(results) = spotify.search_playlist(&query, 50, offset as u32) {
            let mut pls = results.playlists.items.iter().map(|sp| sp.into()).collect();
            let mut r = playlists.write().unwrap();

            if append {
                r.append(&mut pls);
            } else {
                *r = pls;
            }
            return results.playlists.total;
        }
        0
    }

    fn perform_search<I: ListItem>(
        &self,
        handler: SearchHandler<I>,
        results: &Arc<RwLock<Vec<I>>>,
        query: &str,
        //paginator: Option<&Pagination<I>>,
    ) {
        let spotify = self.spotify.clone();
        let query = query.to_owned();
        let results = results.clone();
        let ev = self.events.clone();
        //let paginator = paginator.cloned();

        std::thread::spawn(move || {
            let total_items = handler(&spotify, &results, &query, 0, false) as usize;

            // register paginator if the API has more than one page of results
            //if let Some(mut paginator) = paginator {
                if total_items > results.read().unwrap().len() {
                    let ev = ev.clone();

                    // paginator callback
                    let cb = move |items: Arc<RwLock<Vec<I>>>| {
                        let offset = items.read().unwrap().len();
                        handler(&spotify, &results, &query, offset, true);
                        ev.trigger();
                    };
                    // paginator.set(total_items, Box::new(cb));
                } else {
                    // paginator.clear()
                }
            //}
            ev.trigger();
        });
    }

    pub fn run_search<S: Into<String>>(&mut self, query: S) {
        let query = query.into();

        self.edit_focused = false;

        {
            let query = query.clone();
            //self.edit
            //    .call_on(&Selector::Name(EDIT_ID), |v: &mut EditView| {
            //        v.set_content(query);
            //    });
        }

        // check if API token refresh is necessary before commencing multiple
        // requests to avoid deadlock, as the parallel requests might
        // simultaneously try to refresh the token
        self.spotify.refresh_token();

        // is the query a Spotify URI?
        if let Some(uritype) = URIType::from_uri(&query) {
            // Clear the results if we are going to process a Spotify URI. We need
            // to do this since we are only calling the search function for the
            // given URI type which leaves the previous search results intact.
            let results_tracks = self.results_tracks.clone();
            *results_tracks.write().unwrap() = Vec::new();
            let results_albums = self.results_albums.clone();
            *results_albums.write().unwrap() = Vec::new();
            let results_artists = self.results_artists.clone();
            *results_artists.write().unwrap() = Vec::new();
            let results_playlists = self.results_playlists.clone();
            *results_playlists.write().unwrap() = Vec::new();

            //let mut tab_view = self.tabs.get_mut();
            match uritype {
                URIType::Track => {
                    println!("found track");
                    self.perform_search(
                        Box::new(Self::get_track),
                        &self.results_tracks,
                        &query,
                    );
                    // tab_view.move_focus_to(0);
                }
                URIType::Album => {
                    println!("found album");
                    self.perform_search(
                        Box::new(Self::get_album),
                        &self.results_albums,
                        &query,
                    );
                    // tab_view.move_focus_to(1);
                }
                URIType::Artist => {
                    println!("found artist");
                    self.perform_search(
                        Box::new(Self::get_artist),
                        &self.results_artists,
                        &query,
                    );
                    // tab_view.move_focus_to(2);
                }
                URIType::Playlist => {
                    println!("found playlist");
                    self.perform_search(
                        Box::new(Self::get_playlist),
                        &self.results_playlists,
                        &query,
                    );
                    // tab_view.move_focus_to(3);
                }
            }
        } else {
            self.perform_search(
                Box::new(Self::search_track),
                &self.results_tracks,
                &query,
            );
            self.perform_search(
                Box::new(Self::search_album),
                &self.results_albums,
                &query,
            );
            self.perform_search(
                Box::new(Self::search_artist),
                &self.results_artists,
                &query,
            );
            self.perform_search(
                Box::new(Self::search_playlist),
                &self.results_playlists,
                &query,
            );
        }
    }
}

// impl View for SearchView {
//     fn draw(&self, printer: &Printer<'_, '_>) {
//         {
//             let printer = &printer
//                 .offset((0, 0))
//                 .cropped((printer.size.x, 1))
//                 .focused(self.edit_focused);
//             self.edit.draw(printer);
//         }

//         let printer = &printer
//             .offset((0, 1))
//             .cropped((printer.size.x, printer.size.y - 1))
//             .focused(!self.edit_focused);
//         self.tabs.draw(printer);
//     }

//     fn layout(&mut self, size: Vec2) {
//         self.edit.layout(Vec2::new(size.x, 1));
//         self.tabs.layout(Vec2::new(size.x, size.y - 1));
//     }

//     fn on_event(&mut self, event: Event) -> EventResult {
//         if event == Event::Key(Key::Esc) || event == Event::Key(Key::Tab) {
//             self.edit_focused = !self.edit_focused;
//             return EventResult::Consumed(None);
//         }

//         if self.edit_focused {
//             self.edit.on_event(event)
//         } else {
//             self.tabs.on_event(event)
//         }
//     }

//     fn call_on_any<'a>(&mut self, selector: &Selector<'_>, callback: AnyCb<'a>) {
//         self.edit.call_on_any(selector, &mut |v| callback(v));
//         self.tabs.call_on_any(selector, &mut |v| callback(v));
//     }

//     fn focus_view(&mut self, selector: &Selector<'_>) -> Result<(), ()> {
//         if let Selector::Name(s) = selector {
//             self.edit_focused = s == &"search_edit";
//             Ok(())
//         } else {
//             Err(())
//         }
//     }
// }

// impl ViewExt for SearchView {
//     fn on_command(&mut self, s: &mut Cursive, cmd: &Command) -> Result<CommandResult, String> {
//         match cmd {
//             Command::Search(query) => self.run_search(query.to_string()),
//             Command::Focus(_) => {
//                 self.edit_focused = true;
//                 self.clear();
//                 return Ok(CommandResult::Consumed(None));
//             }
//             _ => {}
//         }

//         let result = if !self.edit_focused {
//             self.tabs.on_command(s, cmd)?
//         } else {
//             CommandResult::Ignored
//         };

//         if let CommandResult::Ignored = result {
//             if let Command::Move(mode, _) = cmd {
//                 match mode {
//                     MoveMode::Up if !self.edit_focused => {
//                         self.edit_focused = true;
//                         return Ok(CommandResult::Consumed(None));
//                     }
//                     MoveMode::Down if self.edit_focused => {
//                         self.edit_focused = false;
//                         return Ok(CommandResult::Consumed(None));
//                     }
//                     _ => {}
//                 }
//             }
//         }

//         Ok(result)
//     }
// }
