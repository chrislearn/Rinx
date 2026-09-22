//! Sharing Moments with DM contacts, like WeChat's friend circle.
//!
//! An account that opts in publishes [`PROFILE_FIELD`] on its Matrix profile
//! (an MSC4133 extended profile field). Its Rinx then keeps its Moments audience
//! in step with its DM contacts:
//! * a DM contact whose profile carries the field is invited to our timeline, and
//! * a Moments invitation from a DM contact is accepted automatically.
//!
//! Requiring the field on both sides limits this to Rinx users who opted in, so
//! contacts on other Matrix clients never receive a Moments invitation they can't use.
//! A contact who has ever been a member of our timeline (including one we removed)
//! is never re-invited.
use std::{
    collections::{BTreeSet, HashMap},
    sync::Mutex,
    time::{Duration, Instant},
};

use anyhow::Result;
use makepad_widgets::{log, warning};
use matrix_sdk::{
    Client, RoomMemberships,
    ruma::{
        OwnedUserId, UserId,
        api::client::profile::{ProfileFieldName, ProfileFieldValue},
    },
};

use super::backend::Service;

/// The profile field announcing that an account shares Moments with its DM contacts.
pub const PROFILE_FIELD: &str = "rs.robius.rinx.moments";
const PROFILE_VALUE: &str = "dm_contacts";

/// How long after login the background loop first reconciles the audience, and how often after that.
const FIRST_RUN: Duration = Duration::from_secs(60);
const LOOP_INTERVAL: Duration = Duration::from_secs(5 * 60);
/// How long a contact's profile lookup is trusted before asking again.
const PROFILE_TTL: Duration = Duration::from_secs(30 * 60);

/// Cached answers to "does this contact share Moments with DM contacts?".
static SHARES: Mutex<Option<HashMap<OwnedUserId, (bool, Instant)>>> = Mutex::new(None);

/// The users we share a joined direct-message room with.
pub fn dm_contacts(client: &Client, owner: &UserId) -> BTreeSet<OwnedUserId> {
    client
        .joined_rooms()
        .into_iter()
        .filter(|room| !super::is_moments(room))
        .flat_map(|room| room.direct_targets())
        .filter_map(|target| UserId::parse(target.as_str()).ok())
        .filter(|id| id != owner)
        .collect()
}

/// Whether `user` publishes the Moments-sharing profile field.
/// Lookup failures (including servers without extended profiles) count as "no".
async fn shares_with_dm_contacts(client: &Client, user: &UserId) -> bool {
    if let Some(cache) = SHARES.lock().unwrap().as_ref()
        && let Some((shares, at)) = cache.get(user)
        && at.elapsed() < PROFILE_TTL
    {
        return *shares;
    }
    let shares = matches!(
        client
            .account()
            .fetch_profile_field_of(user.to_owned(), ProfileFieldName::from(PROFILE_FIELD))
            .await,
        Ok(Some(value)) if value.value().as_str() == Some(PROFILE_VALUE)
    );
    SHARES
        .lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(user.to_owned(), (shares, Instant::now()));
    shares
}

impl Service {
    /// Turns sharing with DM contacts on or off: saves the preference
    /// and publishes or removes the profile field.
    pub async fn set_share_with_dm_contacts(&self, share: bool) -> Result<()> {
        self.update_preferences(|prefs| prefs.share_with_dm_contacts = share)
            .await?;
        let account = self.client.account();
        if share {
            account
                .set_profile_field(ProfileFieldValue::new(
                    PROFILE_FIELD,
                    serde_json::json!(PROFILE_VALUE),
                )?)
                .await?;
            self.sync_dm_sharing().await?;
        } else {
            account
                .delete_profile_field(ProfileFieldName::from(PROFILE_FIELD))
                .await?;
        }
        Ok(())
    }

    /// Accepts Moments invitations from DM contacts and invites DM contacts who
    /// share the same way. Does nothing unless this account opted in.
    ///
    /// Returns how many contacts were invited.
    pub async fn sync_dm_sharing(&self) -> Result<usize> {
        if !self.preferences().await?.share_with_dm_contacts {
            return Ok(0);
        }
        let contacts = dm_contacts(&self.client, &self.owner);

        for room in self.client.invited_rooms() {
            if !super::is_moments(&room) {
                continue;
            }
            if let Ok(invite) = room.invite_details().await
                && contacts.contains(&invite.inviter_id)
            {
                self.invitation(room.room_id(), true).await?;
            }
        }

        let timeline = self.ensure_timeline().await?;
        let Some(room) = self.client.get_room(&timeline) else {
            return Ok(0);
        };
        // Anyone with any membership record (joined, invited, left, removed) is skipped,
        // so removing a viewer or declining sticks.
        let known: BTreeSet<OwnedUserId> = room
            .members_no_sync(RoomMemberships::all())
            .await?
            .into_iter()
            .map(|m| m.user_id().to_owned())
            .collect();
        let mut invited = 0;
        for contact in contacts.iter().filter(|c| !known.contains(*c)) {
            if shares_with_dm_contacts(&self.client, contact).await {
                self.membership(&timeline, contact, true).await?;
                invited += 1;
            }
        }
        Ok(invited)
    }
}

/// Periodically reconciles the Moments audience with DM contacts while logged in.
/// Spawned alongside the other sync tasks, and aborted with them on logout.
pub async fn dm_sharing_loop() {
    let mut wait = FIRST_RUN;
    loop {
        tokio::time::sleep(wait).await;
        wait = LOOP_INTERVAL;
        if let Some(service) = Service::current() {
            match service.sync_dm_sharing().await {
                Ok(0) => {}
                Ok(n) => log!("Moments: invited {n} DM contact(s) to the audience."),
                Err(e) => warning!("Moments: couldn't sync sharing with DM contacts: {e}"),
            }
        }
    }
}
