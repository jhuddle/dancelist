// Copyright 2022 the dancelist authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    errors::InternalError,
    icalendar::{events_to_calendar, Ics},
    model::{
        dancestyle::DanceStyle,
        event::Event,
        events::{Country, Events},
        filters::Filters,
    },
};
use askama::Template;
use axum::{
    extract::{Extension, Query, TypedHeader},
    headers::Host,
    response::Html,
};
use chrono::{naive, Datelike, NaiveDate};

pub async fn index(
    Extension(events): Extension<Events>,
    Query(mut filters): Query<Filters>,
    TypedHeader(host): TypedHeader<Host>,
) -> Result<Html<String>, InternalError> {
    let has_filters = filters.has_some();

    if host.hostname().contains("balfolk.org") && filters.style.is_none() {
        // Default to only showing Balfolk events.
        filters.style = Some(DanceStyle::Balfolk);
    }

    let countries = events.countries(&filters.with_country(None));
    let styles = events.styles(&filters.with_style(None));
    let cities = if filters.country.is_some() {
        events.cities(&filters.with_city(None))
    } else {
        vec![]
    };
    let events = events.matching(&filters);
    let months = sort_and_group_by_month(events);
    let template = IndexTemplate {
        filters,
        months,
        has_filters,
        countries,
        cities,
        styles,
    };
    Ok(Html(template.render()?))
}

pub async fn index_json(
    Extension(events): Extension<Events>,
    Query(filters): Query<Filters>,
) -> Result<String, InternalError> {
    let mut events = events.matching(&filters);
    events.sort_by_key(|event| event.start_time_sort_key());
    let events = Events::cloned(events);
    Ok(serde_json::to_string(&events)?)
}

pub async fn index_toml(
    Extension(events): Extension<Events>,
    Query(filters): Query<Filters>,
) -> Result<String, InternalError> {
    let mut events = events.matching(&filters);
    events.sort_by_key(|event| event.start_time_sort_key());
    let events = Events::cloned(events);
    Ok(toml::to_string(&events)?)
}

pub async fn index_yaml(
    Extension(events): Extension<Events>,
    Query(filters): Query<Filters>,
) -> Result<String, InternalError> {
    let mut events = events.matching(&filters);
    events.sort_by_key(|event| event.start_time_sort_key());
    let events = Events::cloned(events);
    Ok(serde_yaml::to_string(&events)?)
}

pub async fn index_ics(
    Extension(events): Extension<Events>,
    Query(filters): Query<Filters>,
) -> Result<Ics, InternalError> {
    let mut events = events.matching(&filters);
    events.sort_by_key(|event| event.start_time_sort_key());
    let calendar = events_to_calendar(&events);
    Ok(Ics(calendar))
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    filters: Filters,
    months: Vec<Month>,
    has_filters: bool,
    countries: Vec<Country>,
    cities: Vec<String>,
    styles: Vec<DanceStyle>,
}

struct Month {
    /// The first day of the month.
    start: NaiveDate,
    events: Vec<Event>,
}

impl Month {
    pub fn name(&self) -> String {
        self.start.format("%B %Y").to_string()
    }
}

/// Given a list of events in arbitrary order, sort them in ascending order of start date, then group them by starting month.
fn sort_and_group_by_month(mut events: Vec<&Event>) -> Vec<Month> {
    events.sort_by_key(|event| event.start_time_sort_key());

    let mut months = vec![];
    let mut month = Month {
        start: naive::MIN_DATE,
        events: vec![],
    };
    for event in events {
        if event.start_year() == month.start.year() && event.start_month() == month.start.month() {
            month.events.push(event.to_owned());
        } else {
            if !month.events.is_empty() {
                months.push(month);
            }
            month = Month {
                start: NaiveDate::from_ymd(event.start_year(), event.start_month(), 1),
                events: vec![event.to_owned()],
            };
        }
    }
    if !month.events.is_empty() {
        months.push(month);
    }

    months
}
