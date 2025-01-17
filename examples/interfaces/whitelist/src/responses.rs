use sylvia::cw_schema::cw_serde;

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct AdminListResponse {
    pub admins: Vec<String>,
    pub mutable: bool,
}

#[cfg(any(test, feature = "mt"))]
impl AdminListResponse {
    /// Utility function for converting message to its canonical form, so two messages with
    /// different representation but same semantical meaning can be easily compared.
    ///
    /// It could be encapsulated in custom `PartialEq` implementation, but `PartialEq` is expected
    /// to be quickly, so it seems to be reasonable to keep it as representation-equality, and
    /// canonicalize message only when it is needed
    ///
    /// Example:
    ///
    /// ```
    /// # use whitelist::responses::AdminListResponse;
    ///
    /// let resp1 = AdminListResponse {
    ///   admins: vec!["admin1".to_owned(), "admin2".to_owned()],
    ///   mutable: true,
    /// };
    ///
    /// let resp2 = AdminListResponse {
    ///   admins: vec!["admin2".to_owned(), "admin1".to_owned(), "admin2".to_owned()],
    ///   mutable: true,
    /// };
    ///
    /// assert_eq!(resp1.canonical(), resp2.canonical());
    /// ```
    pub fn canonical(mut self) -> Self {
        self.admins.sort();
        self.admins.dedup();
        self
    }
}
