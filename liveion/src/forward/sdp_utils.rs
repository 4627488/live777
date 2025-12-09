use anyhow::Result;
use webrtc::sdp::SessionDescription;

pub fn remove_vp8_from_sdp(sdp: &str) -> Result<String> {
    let mut session = SessionDescription::unmarshal(&mut std::io::Cursor::new(sdp))?;

    for media in &mut session.media_descriptions {
        if media.media_name.media != "video" {
            continue;
        }

        let mut vp8_payload_types = Vec::new();

        // 1. Find VP8 payload types from rtpmap
        // a=rtpmap:<payload_type> <encoding_name>/<clock_rate>...
        for attr in &media.attributes {
            if attr.key == "rtpmap"
                && let Some(value) = &attr.value
                && value.to_uppercase().contains("VP8")
                && let Some(pt_str) = value.split_whitespace().next()
                && let Ok(pt) = pt_str.parse::<u8>()
            {
                // value format: "96 VP8/90000"
                vp8_payload_types.push(pt);
            }
        }

        if vp8_payload_types.is_empty() {
            continue;
        }

        // 2. Remove VP8 attributes
        media.attributes.retain(|attr| {
            if let Some(value) = &attr.value {
                // Check if attribute starts with one of the VP8 payload types
                // e.g. "96 VP8/90000", "96 ...", "96"
                for pt in &vp8_payload_types {
                    let pt_str = pt.to_string();
                    // Check for rtpmap, fmtp, rtcp-fb which usually start with PT
                    if (attr.key == "rtpmap" || attr.key == "fmtp" || attr.key == "rtcp-fb")
                        && value.starts_with(&pt_str)
                    {
                        // Ensure it's the exact PT (e.g. "96" matches "96 ..." but "9" shouldn't match "96")
                        // Check if followed by space or end of string
                        if value.len() == pt_str.len()
                            || value.chars().nth(pt_str.len()) == Some(' ')
                        {
                            return false;
                        }
                    }
                }
            }
            true
        });

        // 3. Remove payload types from m= line (formats)
        media.media_name.formats.retain(|fmt| {
            if let Ok(pt) = fmt.parse::<u8>() {
                !vp8_payload_types.contains(&pt)
            } else {
                true
            }
        });
    }

    Ok(session.marshal())
}
