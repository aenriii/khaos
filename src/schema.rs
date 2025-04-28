// @generated automatically by Diesel CLI.

diesel::table! {
    elections (uuid) {
        uuid -> Text,
        server_id -> Text,
        poll_message_id -> Text,
        start_time -> Timestamptz,
        end_time -> Timestamptz,
        status -> Text,
    }
}

diesel::table! {
    nominees (id) {
        id -> Int4,
        election_id -> Text,
        user_id -> Text,
        poll_option_text -> Text,
        votes_received -> Nullable<Int4>,
        nomination_status -> Text,
    }
}

diesel::table! {
    servers (id) {
        id -> Text,
        announcements_channel_id -> Nullable<Text>,
        poll_channel_id -> Nullable<Text>,
    }
}

diesel::joinable!(elections -> servers (server_id));
diesel::joinable!(nominees -> elections (election_id));

diesel::allow_tables_to_appear_in_same_query!(
    elections,
    nominees,
    servers,
);
