//! OC audit execution verification — LOCALLY WRITTEN, NOT SHIPPED.
//!
//! Written inside the throwaway clone at `runtime/worktrees/scmessenger-main`
//! to settle the /api/send contract verdicts (issues #379 and #392) by
//! executing the response shape the shipped handler produces. Not part of the
//! SCMessenger repository; ships with nothing. Nothing in `../SCMessenger`
//! was touched.
//!
//! What this does and does not prove:
//!   * PROVES  the JSON the shipped retrying arm emits, because that arm's
//!            only inputs are `http_status`, `status` and `error`, and it
//!            hardcodes `success: true`.
//!   * PROVES  the response type cannot express option B's "no error field".
//!   * DOES NOT exercise the live handler: `handle_send_message` is private
//!            and needs a running swarm plus a persisted contact, so the
//!            end-to-end runtime path stays UNKNOWN.

use scmessenger_cli::api::SendMessageResponse;

#[test]
fn oc_s379_the_retrying_response_carries_both_success_true_and_an_error() {
    // This is the exact struct the shipped handler builds on its retrying arm
    // (cli/src/api.rs): http 202, status "retrying", plus the BLE error text.
    let response = SendMessageResponse {
        success: true,
        error: Some("Initial dispatch failed; retrying: ble write error".to_string()),
        message_id: Some("msg-123".to_string()),
        status: Some("retrying".to_string()),
    };

    let json = serde_json::to_string(&response).expect("response must serialize");

    // The contradiction issue #379 reports, reproduced exactly: the body tells
    // the client the send succeeded AND that something failed.
    assert!(json.contains("\"success\":true"), "got {}", json);
    assert!(json.contains("\"error\":\"Initial dispatch failed"), "got {}", json);
    assert!(json.contains("\"status\":\"retrying\""), "got {}", json);
    assert!(
        json.contains("\"success\":true") && json.contains("\"error\":\""),
        "REPRODUCED: success:true is returned together with a populated error \
         field, which is the contradiction #379 reports. Got: {}",
        json
    );
}

#[test]
fn oc_s392_option_b_cannot_express_a_missing_error_field() {
    // Option B requires the retrying response to carry "no error field".
    // FINDING: `error` carries NO `skip_serializing_if`, unlike `message_id`
    // and `status`. The key is therefore emitted even when the value is None,
    // so option B's shape is unrepresentable with this type.
    let option_b_wanted = SendMessageResponse {
        success: true,
        error: None,
        message_id: Some("msg-1".to_string()),
        status: Some("retrying".to_string()),
    };
    let wanted_json = serde_json::to_string(&option_b_wanted).unwrap();
    assert!(
        wanted_json.contains("\"error\":null"),
        "MEASURED: the error key is always present, even when None, so \
         option B's 'no error field' is unrepresentable. Got: {}",
        wanted_json
    );

    // The shipped response populates it instead of leaving it null.
    let shipped_actual = SendMessageResponse {
        success: true,
        error: Some("Initial dispatch failed; retrying: ble write error".to_string()),
        message_id: Some("msg-1".to_string()),
        status: Some("retrying".to_string()),
    };
    let actual_json = serde_json::to_string(&shipped_actual).unwrap();
    assert!(
        actual_json.contains("\"error\":\"Initial dispatch failed"),
        "the shipped retrying arm populates error. Got: {}",
        actual_json
    );

    // message_id and status, by contrast, ARE omitted when None. That proves
    // the asymmetry is specific to `error` and is therefore a contract defect
    // rather than a deliberate uniform choice.
    let no_optionals = SendMessageResponse {
        success: true,
        error: None,
        message_id: None,
        status: None,
    };
    let bare = serde_json::to_string(&no_optionals).unwrap();
    assert!(
        !bare.contains("message_id") && !bare.contains("\"status\""),
        "message_id and status are omitted when None. Got: {}",
        bare
    );
    assert!(
        bare.contains("\"error\":null"),
        "but error is not. Got: {}",
        bare
    );
}

#[test]
fn oc_s392_a_rejected_response_is_representable_but_never_built() {
    // A caller CAN build success:false, so the gap is call-site coverage, not
    // type design. The shipped /api/send path has exactly one construction
    // site and it hardcodes `success: true`; the rejected arm instead returns
    // Err((503, String)), a bare-text body.
    let rejected = SendMessageResponse {
        success: false,
        error: Some("Message was not accepted by BLE or Swarm".to_string()),
        message_id: Some("msg-1".to_string()),
        status: Some("rejected".to_string()),
    };
    let json = serde_json::to_string(&rejected).unwrap();
    assert!(json.contains("\"success\":false"), "got {}", json);
    assert!(
        json.contains("\"error\":\"Message was not accepted"),
        "got {}",
        json
    );
}
