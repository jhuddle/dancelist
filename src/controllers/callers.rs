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

use crate::{errors::InternalError, model::events::Events};
use askama::Template;
use axum::{extract::Extension, response::Html};

pub async fn callers(Extension(events): Extension<Events>) -> Result<Html<String>, InternalError> {
    let callers = events.callers();
    let template = CallersTemplate { callers };
    Ok(Html(template.render()?))
}

#[derive(Template)]
#[template(path = "callers.html")]
struct CallersTemplate {
    callers: Vec<String>,
}
