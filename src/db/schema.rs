table! {
    games (id) {
        id -> Int4,
        name -> Text,
        description -> Nullable<Text>,
        state -> Int2,
    }
}

table! {
    user_games (id) {
        id -> Int4,
        user_id -> Uuid,
        game_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Uuid,
        name -> Text,
        password -> Text,
    }
}

joinable!(user_games -> games (game_id));
joinable!(user_games -> users (user_id));

allow_tables_to_appear_in_same_query!(
    games,
    user_games,
    users,
);
