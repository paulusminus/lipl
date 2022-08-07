pub const DROP: &[&str] = &[
    include_str!("./sql/drop/001_function_set_members.sql"),
    include_str!("./sql/drop/002_view_membership.sql"),
    include_str!("./sql/drop/003_table_member.sql"),
    include_str!("./sql/drop/004_table_lyric.sql"),
    include_str!("./sql/drop/005_table_playlist.sql"),
];

pub const CREATE: &[&str] = &[
    include_str!("./sql/create/001_table_lyric.sql"),
    include_str!("./sql/create/002_table_playlist.sql"),
    include_str!("./sql/create/003_table_member.sql"),
    include_str!("./sql/create/004_index_member_lyric.sql"),
    include_str!("./sql/create/005_index_member_playlist.sql"),
    include_str!("./sql/create/006_view_membership.sql"),
    include_str!("./sql/create/007_function_set_members.sql"),
];

pub mod crud {
    use tokio_postgres::types::Type;

    pub const UPSERT_LYRIC: &str = include_str!("./sql/crud/upsert_lyric.sql");
    pub const UPSERT_LYRIC_TYPES: &[Type] = &[Type::UUID, Type::TEXT, Type::TEXT];

    pub const UPSERT_PLAYLIST: &str = include_str!("./sql/crud/upsert_playlist.sql");
    pub const UPSERT_PLAYLIST_TYPES: &[Type] = &[Type::UUID, Type::TEXT];
    
    pub const DELETE_LYRIC: &str = include_str!("./sql/crud/delete_lyric.sql");
    pub const DELETE_LYRIC_TYPES: &[Type] = &[Type::UUID];
    
    pub const DELETE_PLAYLIST: &str = include_str!("./sql/crud/delete_playlist.sql");
    pub const DELETE_PLAYLIST_TYPES: &[Type] = &[Type::UUID];
    
    pub const SELECT_LYRICS: &str = include_str!("./sql/crud/select_lyrics.sql");
    pub const SELECT_LYRICS_TYPES: &[Type] = &[];

    pub const SELECT_LYRIC_SUMMARIES: &str = include_str!("./sql/crud/select_lyric_summaries.sql");
    pub const SELECT_LYRIC_SUMMARIES_TYPES: &[Type] = &[];
    
    pub const SELECT_LYRIC_DETAIL: &str = include_str!("./sql/crud/select_lyric_detail.sql");
    pub const SELECT_LYRIC_DETAIL_TYPES: &[Type] = &[Type::UUID];
    
    pub const SELECT_PLAYLIST_SUMMARIES: &str = include_str!("./sql/crud/select_playlist_summaries.sql");
    pub const SELECT_PLAYLIST_SUMMARIES_TYPES: &[Type] = &[];
    
    pub const SELECT_PLAYLIST_SUMMARY: &str = include_str!("./sql/crud/select_playlist_summary.sql");
    pub const SELECT_PLAYLIST_SUMMARY_TYPES: &[Type] = &[Type::UUID];
    
    pub const SELECT_PLAYLIST_MEMBERS: &str = include_str!("./sql/crud/select_playlist_members.sql"); 
    pub const SELECT_PLAYLIST_MEMBERS_TYPES: &[Type] = &[Type::UUID];

    pub const SET_PLAYLIST_MEMBERS: &str = include_str!("./sql/crud/set_members.sql");
    pub const SET_PLAYLIST_MEMBERS_TYPES: &[Type] = &[Type::UUID, Type::UUID_ARRAY];
}