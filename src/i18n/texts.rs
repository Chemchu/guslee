use std::fmt::Display;

macro_rules! texts {
    ($($key:ident),* $(,)?) => {
        pub struct Texts {
            $(pub $key: &'static str,)*
            pub schedule: [&'static str; 24],
        }

        impl Texts {
            pub fn entries(&self) -> Vec<(&'static str, &'static str)> {
                vec![$((stringify!($key), self.$key)),*]
            }
        }
    };
}

texts! {
    site_description,
    language_label,
    nav_home,
    nav_news,
    nav_garden,
    nav_tags,
    nav_chess,
    nav_music,
    nav_games,
    nav_browse,
    menu_toggle,
    footer_source,
    schedule_loading,
    schedule_unknown,
    loading,
    search_placeholder_mobile,
    search_placeholder,
    search_switcher_hint,
    search_clear,
    explorer_label,
    explorer_topics,
    explorer_folders,
    sidebar_resize,
    resize_hint,
    switcher_label,
    switcher_placeholder,
    switcher_empty,
    switcher_help,
    not_found_message,
    not_found_back,
    welcome_error,
    panel_resize,
    panel_label,
    panel_metadata,
    panel_outline,
    outline_empty,
    unresolved_link,
    meta_title,
    meta_description,
    meta_reading_time,
    meta_minutes,
    meta_date,
    meta_last_updated,
    meta_tags,
    meta_linked_mentions,
    meta_no_backlinks,
    meta_unlinked_hint,
    meta_unlinked_mentions,
    meta_source,
    meta_not_found,
    tags_title,
    tags_count,
    tags_description,
    tag_back,
    tag_post_one,
    tag_post_other,
    tag_description,
    news_title,
    news_subtitle,
    news_empty_title,
    news_empty_body,
    news_read_more,
    graph_depth_label,
    graph_depth_hint,
    graph_root_label,
    graph_touch_hint,
    graph_recenter,
    chess_title,
    chess_current_rating,
    chess_peak_rating,
    chess_mode_games,
    chess_total_games,
    chess_win_rate,
    chess_draw_rate,
    chess_loss_rate,
    chess_games_count,
    chess_analysis_title,
    chess_games_reviewed,
    chess_openings,
    chess_as_white,
    chess_as_black,
    chess_played,
    chess_time_recently,
    chess_times_recently,
    chess_no_data,
    chess_recent_games,
    chess_win,
    chess_loss,
    chess_draw,
    chess_unknown,
    chess_unknown_opening,
    chess_played_as,
    chess_white,
    chess_black,
    chess_error,
    music_error,
    music_subscriber,
    music_followers,
    music_top_songs,
    music_top_artists,
    music_last_year,
    music_last_6_months,
    music_last_4_weeks,
    music_artist,
    music_item_error,
    music_artists_error,
    music_tracks_error,
    steam_error,
    steam_offline,
    steam_online,
    steam_busy,
    steam_away,
    steam_snooze,
    steam_looking_to_trade,
    steam_looking_to_play,
    steam_unknown,
    steam_avatar_alt,
    steam_level,
    steam_view_profile,
    steam_member_since,
    steam_country,
    steam_recently_played,
    steam_top_played,
    steam_two_weeks,
    steam_total,
}

pub fn fill(text: &str, value: impl Display) -> String {
    text.replacen("{}", &value.to_string(), 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Locale;

    #[test]
    fn values_fill_the_first_placeholder() {
        assert_eq!(fill("{} posts", 3), "3 posts");
        assert_eq!(fill("Notes tagged #{}", "fog"), "Notes tagged #fog");
        assert_eq!(fill("no placeholder", 1), "no placeholder");
    }

    #[test]
    fn placeholders_match_between_languages() {
        let english = Locale::En.texts().entries();
        let spanish = Locale::Es.texts().entries();

        for ((key, en), (_, es)) in english.into_iter().zip(spanish) {
            assert_eq!(en.matches("{}").count(), es.matches("{}").count(), "{key}");
        }
    }

    #[test]
    fn every_hour_has_an_activity_in_every_language() {
        for locale in Locale::ALL {
            assert!(
                locale
                    .texts()
                    .schedule
                    .iter()
                    .all(|activity| !activity.is_empty())
            );
        }
    }
}
