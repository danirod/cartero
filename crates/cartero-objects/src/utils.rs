// Copyright 2024-2026 the Cartero authors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

/// TODO: convert into a macro?
#[cfg(test)]
pub(crate) mod test {
    use std::sync::{atomic::AtomicBool, Arc, Mutex};

    use glib::object::IsA;
    use glib::object::ObjectExt;

    pub(crate) fn assert_emits_signals<T, F>(obj: &T, signals: &[&str], callback: F)
    where
        T: IsA<glib::Object>,
        F: Fn(),
    {
        let triggers = signals
            .iter()
            .map(|signal| (signal, Arc::new(Mutex::new(AtomicBool::new(false)))))
            .collect::<Vec<_>>();
        let handlers = triggers
            .iter()
            .map(|(signal, trigger)| {
                let trigger_cb = trigger.clone();
                obj.connect_local(signal, true, move |_| {
                    let ab = trigger_cb.lock().unwrap();
                    ab.store(true, std::sync::atomic::Ordering::Relaxed);
                    None
                })
            })
            .collect::<Vec<_>>();
        callback();
        for handler in handlers {
            obj.disconnect(handler);
        }
        for (signal, trigger) in triggers {
            let value = trigger
                .lock()
                .unwrap()
                .load(std::sync::atomic::Ordering::Relaxed);
            assert!(value, "signal {} was not emitted", signal);
        }
    }

    pub(crate) fn assert_emits_signal<T, F>(obj: &T, signal: &str, callback: F)
    where
        T: IsA<glib::Object>,
        F: Fn(),
    {
        let trigger = Arc::new(Mutex::new(AtomicBool::new(false)));
        let trigger_cb = trigger.clone();
        let handler = obj.connect_local(signal, true, move |_| {
            let ab = trigger_cb.lock().unwrap();
            ab.store(true, std::sync::atomic::Ordering::Relaxed);
            None
        });
        callback();
        assert!(
            trigger
                .lock()
                .unwrap()
                .load(std::sync::atomic::Ordering::Relaxed),
            "signal {} was not emitted",
            signal
        );
        obj.disconnect(handler);
    }

    /// TODO: convert into a macro?
    #[allow(unused)]
    pub(crate) fn assert_not_emits_signal<T, F>(obj: &T, signal: &str, callback: F)
    where
        T: IsA<glib::Object>,
        F: Fn(),
    {
        let trigger = Arc::new(Mutex::new(AtomicBool::new(false)));
        let trigger_cb = trigger.clone();
        let handler = obj.connect_local(signal, true, move |_| {
            let ab = trigger_cb.lock().unwrap();
            ab.store(true, std::sync::atomic::Ordering::Relaxed);
            None
        });
        callback();
        assert!(
            !trigger
                .lock()
                .unwrap()
                .load(std::sync::atomic::Ordering::Relaxed),
            "emited signal {}",
            signal
        );
        obj.disconnect(handler);
    }
}
