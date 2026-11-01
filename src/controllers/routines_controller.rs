use actix_web::{
    get,
    web::{self, Html},
};
use chrono::{Timelike, Utc};
use maud::html;

use crate::i18n::{Locale, Texts};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_current_schedule_activity);
}

#[get("/schedule")]
async fn get_current_schedule_activity(locale: Locale) -> Html {
    let html = html! {
        span
        class="italic"
        {
            (schedule_activity(Utc::now().hour(), locale.texts()))
        }
    };

    Html::new(html)
}

fn schedule_activity(hour: u32, texts: &Texts) -> &'static str {
    usize::try_from(hour)
        .ok()
        .and_then(|hour| texts.schedule.get(hour))
        .copied()
        .unwrap_or(texts.schedule_unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_hour_of_the_day_has_an_activity() {
        for locale in Locale::ALL {
            let texts = locale.texts();
            for hour in 0..24 {
                assert_ne!(
                    schedule_activity(hour, texts),
                    texts.schedule_unknown,
                    "hour {hour}"
                );
            }
        }
    }

    #[test]
    fn known_hours_map_to_their_activity() {
        assert_eq!(schedule_activity(4, Locale::En.texts()), "Sleeping");
        assert_eq!(schedule_activity(12, Locale::En.texts()), "Lunch hehe");
        assert_eq!(schedule_activity(12, Locale::Es.texts()), "Comiendo jeje");
    }

    #[test]
    fn out_of_range_hours_fall_back() {
        let texts = Locale::En.texts();
        assert_eq!(schedule_activity(24, texts), texts.schedule_unknown);
    }
}
