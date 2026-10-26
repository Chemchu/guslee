use std::collections::HashMap;

use actix_web::{
    get,
    web::{self, Html},
};
use chrono::{Timelike, Utc};
use maud::html;

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_current_schedule_activity);
}

#[get("/schedule")]
async fn get_current_schedule_activity() -> Html {
    let html = html! {
        span
        class="italic"
        {
            (schedule_activity(Utc::now().hour()))
        }
    };

    Html::new(html)
}

fn schedule_activity(hour: u32) -> &'static str {
    let schedule: HashMap<u32, &'static str> = HashMap::from([
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

    schedule
        .get(&hour)
        .copied()
        .unwrap_or("I should probably see why the hour is not between 0 and 23...")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FALLBACK: &str = "I should probably see why the hour is not between 0 and 23...";

    #[test]
    fn every_hour_of_the_day_has_an_activity() {
        for hour in 0..24 {
            assert_ne!(schedule_activity(hour), FALLBACK, "hour {hour}");
        }
    }

    #[test]
    fn known_hours_map_to_their_activity() {
        assert_eq!(schedule_activity(4), "Sleeping");
        assert_eq!(schedule_activity(12), "Lunch hehe");
    }

    #[test]
    fn out_of_range_hours_fall_back() {
        assert_eq!(schedule_activity(24), FALLBACK);
    }
}
