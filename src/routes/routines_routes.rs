use std::collections::HashMap;

use actix_web::{get, web::Html};
use chrono::{Timelike, Utc};
use maud::html;

#[get("/schedule")]
pub async fn get_current_schedule_activity() -> Html {
    let current_hour = Utc::now().hour();
    let schedule: HashMap<u32, &str> = HashMap::from([
        (0, "Coding, probably"),
        (1, "Coding, probably"),
        (2, "Sleeping, I hope"),
        (3, "Sleeping, I hope"),
        (4, "Sleeping"),
        (5, "Sleeping"),
        (6, "Sleeping"),
        (7, "Sleeping"),
        (8, "Waking up for work"),
        (9, "Working..."),
        (10, "Working..."),
        (11, "Working..."),
        (12, "Lunch hehe"),
        (13, "Working..."),
        (14, "Probably on my fourth coffee of the day"),
        (15, "Still working, yep"),
        (16, "..working, but also looking at the clock"),
        (17, "Finally off work, going back to bed"),
        (18, "Maybe playing chess, maybe seeing funny videos"),
        (19, "Playing something"),
        (20, "Playing something"),
        (21, "Having dinner while talking to Bea"),
        (22, "Talking to Bea hehe"),
        (23, "Coding again, probably"),
    ]);

    let html = html! {
        span
        class="italic"
        {
            (schedule.get(&current_hour).unwrap_or_else(|| &"I should probably see why the hour is not between 0 and 23..."))
        }
    };

    Html::new(html)
}
