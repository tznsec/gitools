//! Rejestr szablonów.

mod astro;
mod common;
mod dotnet;
mod electron;
mod empty;
mod flutter;
mod go;
mod node;
mod python;
mod react;
mod rust_tpl;
mod svelte;
mod tauri;
mod vue;

use crate::template::Template;

pub fn all() -> Vec<Template> {
    vec![
        empty::template(),
        rust_tpl::template(),
        node::template(),
        react::template(),
        vue::template(),
        svelte::template(),
        astro::template(),
        electron::template(),
        tauri::template(),
        flutter::template(),
        dotnet::template(),
        python::template(),
        go::template(),
    ]
}