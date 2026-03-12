// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{any::TypeId, borrow::Borrow, collections::BTreeMap};

use serde::{Deserialize, Serialize};

use crate::route::{RouteEntry, RouteMetadata};

mod history;
use history::*;

pub struct Router {
    route_id_cache: BTreeMap<TypeId, RouteId>,
    routes: BTreeMap<RouteId, RegisteredRoute>,
    history: NavHistory,
    on_navigation_start: Vec<Box<dyn Fn(&NavHistory)>>,
    on_navigation_end: Vec<Box<dyn Fn(&NavHistory)>>,
    enabled: bool,
}

impl Default for Router {
    fn default() -> Self {
        Self {
            route_id_cache: Default::default(),
            routes: Default::default(),
            history: Default::default(),
            on_navigation_start: Default::default(),
            on_navigation_end: Default::default(),
            enabled: true,
        }
    }
}

impl Router {
    /// Notification callback for navigation start. Fires before the navigation is complete.
    pub fn register_on_navigation_start(&mut self, func: impl Fn(&NavHistory) + 'static) {
        self.on_navigation_start.push(Box::new(func));
    }

    /// Notification callback for navigation end. Fires after the navigation is complete.
    pub fn register_on_navigation_end(&mut self, func: impl Fn(&NavHistory) + 'static) {
        self.on_navigation_end.push(Box::new(func));
    }

    /// Pushes a new route to the navigation history and calls the registered callback.
    ///
    /// # Returns
    /// - `true` if the route was pushed and navigation started
    /// - `false` if the route is already active (exact match)
    ///
    /// # Panics
    /// Panics if the route type has not been registered with [`Self::register_route`].
    pub fn push_route<T>(&mut self, route: T) -> bool
    where
        T: RouteEntry + 'static,
    {
        self.try_push_route(route).unwrap_or_else(|| {
            panic!("Route {} does not exist", T::route_id());
        })
    }

    /// Attempts to push a new route to the navigation history and calls the registered callback.
    ///
    /// # Returns
    /// - `Some(true)` if the route was pushed and navigation started
    /// - `Some(false)` if the route is already active (exact match)
    /// - `None` if the route type has not been registered with [`Self::register_route`].
    pub fn try_push_route<T>(&mut self, route: T) -> Option<bool>
    where
        T: RouteEntry + 'static,
    {
        if let Some(registered_route) = self.get_registered_route::<T>() {
            let path = route.ser_route().expect("Route can be serialized");
            let history_entry = HistoryEntry::new(path, route);
            Some(self.push_history_entry(registered_route.callback, history_entry))
        } else {
            None
        }
    }

    /// push a route as a raw string
    pub fn push_raw_route(&mut self, path: String) -> Result<bool, RawRouteError> {
        let (callback, history_entry) = self.create_raw_history_entry(path)?;
        Ok(self.push_history_entry(callback, history_entry))
    }

    /// Replaces the current route in the history with the new route.
    ///
    /// # Returns
    /// - `true` if the route was replaced and navigation started
    /// - `false` if there is no current route to replace
    ///
    /// # Panics
    /// Panics if the route type has not been registered with [`Self::register_route`].
    pub fn replace_route<T>(&mut self, route: T) -> bool
    where
        T: RouteEntry + 'static,
    {
        self.try_replace_route(route).unwrap_or_else(|| {
            panic!("Route {} does not exist", T::route_id());
        })
    }

    /// Attempts to replace the current route in the history with the new route.
    ///
    /// # Returns
    /// - `Some(true)` if the route was replaced and navigation started
    /// - `Some(false)` if there is no current route to replace
    /// - `None` if the route type has not been registered with [`Self::register_route`].
    pub fn try_replace_route<T>(&mut self, route: T) -> Option<bool>
    where
        T: RouteEntry + 'static,
    {
        if let Some(route_entry) = self.get_registered_route::<T>() {
            let path = route.ser_route().expect("Route can be serialized");
            let history_entry = HistoryEntry::new(path, route);
            Some(self.replace_history_entry(route_entry.callback, history_entry))
        } else {
            None
        }
    }

    /// Replaces the current route in the history with the new route.
    pub fn replace_raw_route(&mut self, path: String) -> Result<bool, RawRouteError> {
        let (callback, history_entry) = self.create_raw_history_entry(path)?;
        Ok(self.replace_history_entry(callback, history_entry))
    }

    /// Execute a function with access to the current history.
    pub fn with_history<F, R>(&self, func: F) -> R
    where
        F: FnOnce(&NavHistory) -> R,
    {
        func(&self.history)
    }

    /// Navigate backward in the history.
    /// Returns true if the navigation was successful.
    /// if the backward stack is of length 1, returns false.
    pub fn navigate_backward(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        let result = self.history.nav_backward();
        if result {
            self.on_navigation_start();
            self.call_current();
            self.on_navigation_end();
        }
        result
    }

    /// Navigate forward in the history.
    /// Returns true if the navigation was successful.
    /// if the forward stack is empty, returns false.
    pub fn navigate_forward(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        let result = self.history.nav_forward();
        if result {
            self.on_navigation_start();
            self.call_current();
            self.on_navigation_end();
        }
        result
    }

    /// if router can navigate forward.
    pub fn has_forward(&self) -> bool { self.enabled && self.history.has_forward() }

    /// if router can navigate backward.
    pub fn has_back(&self) -> bool { self.enabled && self.history.has_backward() }

    pub fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled }

    pub fn is_enabled(&self) -> bool { self.enabled }

    // gets the current route path as a string
    pub fn get_active_raw(&mut self) -> Option<String> {
        let current = self.history.get_current_mut()?;
        let path = current.path.clone();
        Some(path)
    }

    pub fn clear_history(&mut self) { self.history.clear() }
}

impl Router {
    #[doc(hidden)]
    pub fn new() -> Self { Self::default() }

    #[doc(hidden)]
    pub fn register_route<T>(&mut self, func: impl Fn(&mut Router) + 'static)
    where
        T: RouteEntry + 'static,
    {
        use std::collections::btree_map::Entry;

        let route_id = RouteId::new::<T>();
        let entry = self.routes.entry(route_id.clone());
        match entry {
            Entry::Occupied(e) => {
                panic!("Duplicate route ID {:?}", e.key());
            }
            Entry::Vacant(entry) => {
                self.route_id_cache.insert(TypeId::of::<T>(), route_id);
                let registered_route = RegisteredRoute::new::<T>(func);
                entry.insert(registered_route);
            }
        }
    }

    #[doc(hidden)]
    pub fn with_active<T, F, R>(&mut self, func: F) -> Option<R>
    where
        T: RouteEntry + 'static,
        F: FnOnce(&T) -> R,
    {
        let current = self.history.get_current_mut()?;
        let value = current.get_value::<T>()?;
        Some(func(value))
    }

    #[doc(hidden)]
    pub fn get_active<T>(&mut self) -> Option<T>
    where
        T: RouteEntry + Clone + 'static,
    {
        self.with_active(Clone::clone)
    }

    fn get_registered_route<T>(&self) -> Option<&RegisteredRoute>
    where
        T: RouteEntry + 'static,
    {
        let route_id = self.route_id_cache.get(&TypeId::of::<T>());
        route_id.and_then(|id| self.routes.get(id))
    }

    fn push_history_entry(&mut self, callback: RouteCallback, entry: HistoryEntry) -> bool {
        // Push failed because route is already active.
        if self.history.push(entry).is_some() {
            false
        } else {
            self.on_navigation_start();
            callback(self);
            self.on_navigation_end();
            true
        }
    }

    fn replace_history_entry(&mut self, callback: RouteCallback, entry: HistoryEntry) -> bool {
        if self.history.replace(entry) {
            self.on_navigation_start();
            callback(self);
            self.on_navigation_end();
            true
        } else {
            false
        }
    }

    fn create_raw_history_entry(&self, path: String) -> Result<(RouteCallback, HistoryEntry), RawRouteError> {
        // todo: this matching should be more efficient.
        let route = self
            .routes
            .iter()
            .find(|(_id, route)| route.matches(&path))
            .map(|(_, route)| route)
            .ok_or(RawRouteError::NotFound)?;

        let data = (route.deserialize)(&path).map_err(RawRouteError::Deserialize)?;

        let history_entry = HistoryEntry::new_raw(route.id.clone(), path, data);

        Ok((route.callback, history_entry))
    }

    fn on_navigation_start(&self) {
        for func in self.on_navigation_start.iter() {
            func(&self.history);
        }
    }

    fn on_navigation_end(&self) {
        for func in self.on_navigation_end.iter() {
            func(&self.history);
        }
    }

    fn call_current(&mut self) {
        let current = self.history.get_current_mut().unwrap();
        let route_id = current.route.clone();
        let route = self.routes.get(&route_id).expect("Route to exist");
        (route.callback)(self);
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RouteId {
    pub id: String,
}

impl RouteId {
    fn new<T: RouteEntry>() -> Self { Self { id: T::route_id() } }
}

impl std::fmt::Debug for RouteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.id) }
}

impl Borrow<str> for RouteId {
    fn borrow(&self) -> &str { &self.id }
}

#[derive(Debug)]
pub enum RawRouteError {
    NotFound,
    Deserialize(crate::route::de::Error),
}

struct RegisteredRoute {
    id: RouteId,
    metadata: RouteMetadata,
    deserialize: fn(&str) -> Result<Box<dyn CachedEntry>, crate::route::de::Error>,
    callback: RouteCallback,
}

type RouteCallback = &'static dyn Fn(&mut Router);

impl Drop for RegisteredRoute {
    fn drop(&mut self) {
        let _callback = unsafe {
            // Convert &'static to *mut by:
            // 1. First cast to *const
            // 2. Then cast to *mut
            let ptr = self.callback as *const _;
            let ptr = ptr as *mut dyn Fn(&mut Router);
            Box::from_raw(ptr)
        };
    }
}

impl RegisteredRoute {
    fn new<T: RouteEntry + 'static>(callback: impl Fn(&mut Router) + 'static) -> Self {
        fn route_entry_deserialize<T>(path: &str) -> Result<Box<dyn CachedEntry>, crate::route::de::Error>
        where
            T: RouteEntry + 'static,
        {
            let value = T::de_route(path)?;
            Ok(Box::new(CachedEntryImpl { value }))
        }

        Self {
            id: RouteId::new::<T>(),
            metadata: T::metadata(),
            deserialize: route_entry_deserialize::<T>,
            callback: Box::leak::<'static>(Box::new(callback)),
        }
    }

    fn matches(&self, path: &str) -> bool { self.metadata.full_matches(path) }
}

#[cfg(test)]
mod test {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::*;
    use crate::route;
    use crate::route::*;

    #[route(path = "/user/{user_id}")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct PathOne {
        user_id: u32,
    }

    #[route(path = "/settings/{settings_id}")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct PathTwo {
        settings_id: String,
    }

    #[route(path = "/home?{is-developer}")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct PathThree {
        is_developer: bool,
    }

    #[route(path = "/redirect?{redirect_url}")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct PathFour {
        redirect_url: String,
    }

    fn init_router() -> Router {
        let mut router = Router::new();
        router.register_route::<PathOne>(|_| ());
        router.register_route::<(PathOne, PathTwo)>(|_| ());
        router.register_route::<(PathOne, PathThree)>(|_| ());
        router.register_route::<(PathThree, PathFour)>(|_| ());
        router
    }

    #[test]
    fn raw_route() {
        let mut router = init_router();

        // push

        assert!(router.push_raw_route("/user/123".to_string()).is_ok());
        assert_eq!(router.get_active::<PathOne>(), Some(PathOne { user_id: 123 }));
        assert!(router.push_raw_route("/user/123/settings/about".to_string()).is_ok());
        assert_eq!(
            router.get_active::<(PathOne, PathTwo)>(),
            Some((PathOne { user_id: 123 }, PathTwo { settings_id: "about".into() }))
        );

        assert!({
            let result = router.push_raw_route("/user/notanumber".to_string());
            matches!(result, Err(RawRouteError::Deserialize(_)))
        });
        assert!({
            let result = router.push_raw_route("not a match".to_string());
            matches!(result, Err(RawRouteError::NotFound))
        });

        assert_eq!(router.history.backward.len(), 2);

        // replace

        assert!(router.replace_raw_route("/user/456".to_string()).is_ok());
        assert_eq!(router.get_active::<PathOne>(), Some(PathOne { user_id: 456 }));
        assert_eq!(router.history.backward.len(), 2);

        assert!(router.replace_raw_route("/user/789/settings/profile".to_string()).is_ok());
        assert_eq!(
            router.get_active::<(PathOne, PathTwo)>(),
            Some((PathOne { user_id: 789 }, PathTwo { settings_id: "profile".into() }))
        );
        assert_eq!(router.history.backward.len(), 2);

        assert!({
            let result = router.replace_raw_route("/user/notanumber".to_string());
            matches!(result, Err(RawRouteError::Deserialize(_)))
        });
        assert!({
            let result = router.replace_raw_route("not a match".to_string());
            matches!(result, Err(RawRouteError::NotFound))
        });

        // query params

        let query_route = (PathOne { user_id: 0 }, PathThree { is_developer: true }).ser_route().unwrap();
        assert_eq!(query_route, "/user/0/home?is_developer=true");
        assert!(router.push_raw_route(query_route).is_ok());
        assert_eq!(
            router.get_active::<(PathOne, PathThree)>(),
            Some((PathOne { user_id: 0 }, PathThree { is_developer: true }))
        );

        let redirect_route =
            (PathThree { is_developer: true }, PathFour { redirect_url: "https://example.com".into() })
                .ser_route()
                .unwrap();
        assert_eq!(redirect_route, "/home/redirect?is_developer=true&redirect_url=https%3A%2F%2Fexample.com");
        assert!(router.push_raw_route(redirect_route).is_ok());
        assert_eq!(
            router.get_active::<(PathThree, PathFour)>(),
            Some((PathThree { is_developer: true }, PathFour { redirect_url: "https://example.com".into() }))
        );
    }

    #[test]
    fn navigation() {
        let mut router = init_router();

        let one = PathOne { user_id: 123 };
        let two = PathTwo { settings_id: "about".into() };

        // push

        assert!(router.push_route(one.clone()));
        assert_eq!(router.get_active::<PathOne>(), Some(one.clone()));

        assert!(router.push_route((one.clone(), two.clone())));
        assert_eq!(router.get_active::<(PathOne, PathTwo)>(), Some((one.clone(), two.clone())));

        // navigate backward/forward

        assert!(router.navigate_backward());
        assert_eq!(router.get_active::<PathOne>(), Some(one.clone()));

        // no more backward
        assert!(!router.navigate_backward());

        assert!(router.navigate_forward());
        assert_eq!(router.get_active::<(PathOne, PathTwo)>(), Some((one.clone(), two.clone())));

        // no more forward
        assert!(!router.navigate_forward());

        // replace

        let new_one = PathOne { user_id: 456 };
        assert!(router.replace_route((new_one.clone(), two.clone())));
        assert_eq!(router.get_active::<(PathOne, PathTwo)>(), Some((new_one.clone(), two.clone())));

        assert_eq!(router.history.backward.len(), 2);
        assert!(router.history.forward.is_empty());

        // Clear de-serialized values.

        router.history.backward.iter_mut().for_each(|b| b.value = None);
        router.history.forward.iter_mut().for_each(|b| b.value = None);

        assert!(router.navigate_backward());
        assert_eq!(router.get_active::<PathOne>(), Some(one.clone()));

        assert!(router.navigate_forward());
        assert_eq!(router.get_active::<(PathOne, PathTwo)>(), Some((new_one.clone(), two.clone())));

        let history = &router.history;
        println!("{history:#?}");
    }

    #[test]
    fn raw_string_get_and_return() {
        let mut router = init_router();
        assert!(router.push_raw_route("/user/123".to_string()).is_ok());
        let initial_route = router.get_active_raw().unwrap();
        assert_eq!(initial_route, "/user/123");

        assert!(router.push_raw_route("/user/123/settings/about".to_string()).is_ok());
        assert_eq!(router.get_active_raw().unwrap(), "/user/123/settings/about");

        assert!(router.push_raw_route(initial_route).is_ok());
        assert_eq!(router.get_active_raw(), Some("/user/123".to_string()));
    }

    #[test]
    fn on_nav_start_and_end_execution_order() {
        let mut router = Router::new();
        router.register_route::<PathOne>(|_| {});

        let on_nav_start_count = Rc::new(Cell::new(0));
        let on_nav_end_count = Rc::new(Cell::new(0));

        router.register_on_navigation_start({
            let on_nav_start_count = on_nav_start_count.clone();
            let on_nav_end_count = on_nav_end_count.clone();
            move |_| {
                assert_eq!(on_nav_end_count.get(), 0);
                on_nav_start_count.set(on_nav_start_count.get() + 1);
            }
        });

        router.register_on_navigation_end({
            let on_nav_start_count = on_nav_start_count.clone();
            let on_nav_end_count = on_nav_end_count.clone();
            move |_| {
                assert_eq!(on_nav_start_count.get(), 1);
                on_nav_end_count.set(on_nav_end_count.get() + 1);
            }
        });

        assert!(router.push_route(PathOne { user_id: 123 }));
        assert_eq!(on_nav_start_count.get(), 1);
        assert_eq!(on_nav_end_count.get(), 1);
    }

    #[test]
    fn on_nav_start() {
        let mut router = Router::new();
        let value = Rc::new(Cell::new(0));

        router.register_route::<PathOne>(|_| {});

        router.register_route::<(PathOne, PathTwo)>(|_| {});

        router.register_on_navigation_start({
            let value = value.clone();
            move |_history| {
                value.set(value.get() + 1);
            }
        });

        assert!(router.push_route(PathOne { user_id: 123 }));
        assert!(router.replace_route(PathOne { user_id: 456 }));

        // Test raw route push
        assert!(router.push_raw_route("/user/789".to_string()).is_ok());
        assert!(router.replace_raw_route("/user/101/settings/profile".to_string()).is_ok());

        assert_eq!(value.get(), 4);
    }

    #[test]
    fn on_nav_end() {
        use std::cell::Cell;
        use std::rc::Rc;

        let mut router = Router::new();
        let value = Rc::new(Cell::new(0));

        router.register_route::<PathOne>(|_| {});
        router.register_route::<(PathOne, PathTwo)>(|_| {});

        router.register_on_navigation_end({
            let value = value.clone();
            move |_| {
                value.set(value.get() + 1);
            }
        });

        assert!(router.push_route(PathOne { user_id: 123 }));
        assert_eq!(value.get(), 1);
        assert!(router.replace_route(PathOne { user_id: 456 }));
        assert_eq!(value.get(), 2);
        assert!(router.push_raw_route("/user/789".to_string()).is_ok());
        assert_eq!(value.get(), 3);
        assert!(router.replace_raw_route("/user/101/settings/profile".to_string()).is_ok());
        assert_eq!(value.get(), 4);
    }

    #[test]
    fn test_query_routes() {
        let mut router = Router::new();
        router.register_route::<PathThree>(|_| ());
        router.register_route::<(PathThree, PathFour)>(|_| ());

        // valid query parameters
        let push_result = router.push_raw_route("/home?is_developer=true".to_string());
        assert!(push_result.is_ok(), "push_result: {push_result:?}");
        assert_eq!(router.get_active::<PathThree>(), Some(PathThree { is_developer: true }));

        assert!(router.push_raw_route("/home?is_developer=false".to_string()).is_ok());
        assert_eq!(router.get_active::<PathThree>(), Some(PathThree { is_developer: false }));

        // invalid query parameter values
        assert!({
            let result = router.push_raw_route("/home?is_developer=notabool".to_string());
            matches!(result, Err(RawRouteError::Deserialize(_)))
        });

        // missing query parameter
        let result = router.push_raw_route("/home".to_string());
        assert!(result.is_ok());

        // with malformed query string
        let result = router.push_raw_route("/home?".to_string());
        assert!(result.is_ok());

        // double query parameters
        let result =
            router.push_raw_route("/home/redirect?is_developer=true&redirect_url=onetwothree".to_string());
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(
            router.get_active::<(PathThree, PathFour)>(),
            Some((PathThree { is_developer: true }, PathFour { redirect_url: "onetwothree".into() }))
        );

        // one query param missing
        let result = router.push_raw_route("/home/redirect?redirect_url=onetwothree".to_string());
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(
            router.get_active::<(PathThree, PathFour)>(),
            Some((PathThree { is_developer: false }, PathFour { redirect_url: "onetwothree".into() }))
        );

        // multiple query params missing
        let result = router.push_raw_route("/home/redirect".to_string());
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(
            router.get_active::<(PathThree, PathFour)>(),
            Some((PathThree { is_developer: false }, PathFour { redirect_url: "".into() }))
        );
    }

    #[test]
    fn try_push_replace_route() {
        let mut router = Router::new();
        router.register_route::<PathOne>(|_| ());

        assert!(router.try_push_route(PathOne { user_id: 123 }).is_some());
        assert!(router.try_replace_route(PathOne { user_id: 456 }).is_some());
        assert!(router.try_push_route(PathTwo { settings_id: "one".into() }).is_none());
        assert!(router.try_replace_route(PathTwo { settings_id: "two".into() }).is_none());

        router.register_route::<PathTwo>(|_| ());
        assert!(router.try_push_route(PathOne { user_id: 456 }).is_some());
        assert!(router.try_push_route(PathTwo { settings_id: "one".into() }).is_some());
        assert!(router.try_replace_route(PathOne { user_id: 678 }).is_some());
        assert!(router.try_replace_route(PathTwo { settings_id: "two".into() }).is_some());
    }
}
