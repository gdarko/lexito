use crate::client::TranslationRequest;

pub fn system_prompt(custom: Option<&str>) -> String {
    let base = concat!(
        "You translate gettext catalog entries. Return JSON only. ",
        "Preserve placeholders, named tokens, and any HTML/XML-like tags exactly. ",
        "Do not explain your answer.\n\n",
        "When given a single entry, return: {\"singular\":\"...\",\"plurals\":[\"...\"]}\n",
        "When given multiple entries, return a JSON object:\n",
        "{\"translations\":[{\"id\":\"1\",\"singular\":\"...\",\"plurals\":[...]}, ...]}",
    );

    match custom.filter(|value| !value.trim().is_empty()) {
        Some(custom) => format!("{base}\n\nAdditional instructions:\n{custom}"),
        None => base.to_string(),
    }
}

pub fn user_prompt(request: &TranslationRequest) -> String {
    let mut body = format!(
        "Target locale: {}\nSource singular:\n{}\n",
        request.target_locale, request.msgid
    );

    if let Some(msgid_plural) = &request.msgid_plural {
        body.push_str(&format!("Source plural:\n{msgid_plural}\n"));
    }

    if let Some(msgctxt) = &request.msgctxt {
        body.push_str(&format!("Context:\n{msgctxt}\n"));
    }

    if !request.comments.is_empty() {
        body.push_str("Comments:\n");
        for comment in &request.comments {
            body.push_str("- ");
            body.push_str(comment);
            body.push('\n');
        }
    }

    if !request.references.is_empty() {
        body.push_str("References:\n");
        for reference in &request.references {
            body.push_str("- ");
            body.push_str(reference);
            body.push('\n');
        }
    }

    body.push_str(
        "Return valid JSON in this shape:\n{\"singular\":\"...\",\"plurals\":[\"...\"]}\n",
    );

    body
}

pub fn batch_user_prompt(requests: &[TranslationRequest]) -> String {
    let locale = requests
        .first()
        .map(|r| r.target_locale.as_str())
        .unwrap_or("en");

    let mut body = format!("Target locale: {locale}\nTranslate these entries:\n\n");

    for (i, request) in requests.iter().enumerate() {
        let id = i + 1;
        body.push_str(&format!("[{id}] msgid: {:?}\n", request.msgid));

        if let Some(msgid_plural) = &request.msgid_plural {
            body.push_str(&format!("    plural: {msgid_plural:?}\n"));
        }
        if let Some(msgctxt) = &request.msgctxt {
            body.push_str(&format!("    context: {msgctxt:?}\n"));
        }
        if !request.comments.is_empty() {
            let joined = request.comments.join("; ");
            body.push_str(&format!("    comment: {joined}\n"));
        }
    }

    body.push_str(&format!(
        "\nReturn a JSON object with a \"translations\" array of {count} objects in order:\n\
         {{\"translations\":[{{\"id\":\"1\",\"singular\":\"...\"}}, ...]}}\n\
         Include \"plurals\":[...] only if the source has a plural form.\n",
        count = requests.len()
    ));

    body
}
