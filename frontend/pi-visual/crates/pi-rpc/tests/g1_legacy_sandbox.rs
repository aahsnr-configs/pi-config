//! ARCHIVED P03 evidence: original sandbox/combined requirement, excluded from revised G1.
//! Run only explicitly by --test g1_legacy_sandbox; no production dependency.
use pi_rpc::{
    client::{Client, Limits},
    extension_ui::Answer,
};
use serde_json::{Value, json};
use std::{
    os::unix::fs::{DirBuilderExt, symlink},
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
#[tokio::test]
#[ignore = "explicit installed Pi/security packages + loopback + OS sandbox prerequisites"]
async fn archived_original_security_negative_suite() {
    let exe = std::env::var("PI_VISUAL_TEST_PI").expect("explicit Pi path");
    let packages = PathBuf::from(
        std::env::var("PI_VISUAL_TEST_PACKAGES").expect("explicit installed package directory"),
    );
    for case in [
        "combined-allow",
        "combined-bash-allow",
        "combined-deny",
        "combined-hard",
        "sandbox-allow",
        "sandbox-bash-allow",
        "sandbox-read",
        "sandbox-symlink",
        "sandbox-traversal",
        "sandbox-bash",
        "sandbox-deny",
        "sandbox-hard",
        "permission-allow",
        "permission-bash-allow",
        "permission-read",
        "permission-hard",
        "permission-deny",
        "permission-symlink",
        "permission-traversal",
        "permission-bash",
        "permission-ask-deny",
        "permission-ask-cancel",
        "permission-ask-timeout",
        "permission-ask-approve",
    ] {
        let root = std::env::temp_dir().join(format!(
            "pi-visual-g1-{case}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        for d in ["home", "agent", "project", "outside", "cache", "config"] {
            std::fs::create_dir(root.join(d)).unwrap();
        }
        let inside = root.join("project/allowed.txt");
        let outside = root.join("outside/marker.txt");
        let hard = root.join("project/hard.txt");
        for p in [&inside, &outside, &hard] {
            std::fs::write(p, "unchanged").unwrap();
        }
        symlink(&outside, root.join("project/link.txt")).unwrap();
        let target =
            if case.ends_with("allow") || case.ends_with("approve") || case.ends_with("read") {
                inside.clone()
            } else if case.ends_with("hard") {
                hard.clone()
            } else if case.ends_with("symlink") {
                root.join("project/link.txt")
            } else if case.ends_with("traversal") {
                root.join("project/../outside/marker.txt")
            } else {
                outside.clone()
            };
        let bash = case.contains("bash");
        let plan = if bash {
            json!({"name":"bash","arguments":{"command":format!("printf changed > '{}'",target.display())}})
        } else if case.ends_with("read") {
            json!({"name":"read","arguments":{"path":target}})
        } else {
            json!({"name":"write","arguments":{"path":target,"content":"changed"}})
        };
        let mut provider = Command::new("python3")
            .env_clear()
            .env("PI_VISUAL_TOOL_PLAN", plan.to_string())
            .args([
                "-u",
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/fixtures/security-provider.py"
                ),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut port = String::new();
        tokio::time::timeout(
            Duration::from_secs(3),
            BufReader::new(provider.stdout.take().unwrap()).read_line(&mut port),
        )
        .await
        .unwrap()
        .unwrap();
        let port: u16 = port.trim().parse().expect("loopback fixture startup");
        std::fs::write(root.join("agent/models.json"),json!({"providers":{"fixture":{"baseUrl":format!("http://127.0.0.1:{port}/v1"),"api":"openai-completions","apiKey":"local-test-only","models":[{"id":"fixture","reasoning":false,"input":["text"],"contextWindow":32768,"maxTokens":256,"cost":{"input":0,"output":0,"cacheRead":0,"cacheWrite":0}}]}}}).to_string()).unwrap();
        std::fs::write(root.join("agent/sandbox.json"),json!({"enabled":true,"filesystem":{"allowRead":[root.join("project")],"allowWrite":[root.join("project")],"denyRead":[],"denyWrite":[hard]},"network":{"allowedDomains":["127.0.0.1","localhost"],"deniedDomains":[]}}).to_string()).unwrap();
        std::fs::create_dir_all(root.join("agent/extensions/pi-permission-system")).unwrap();
        let ask = case.contains("ask");
        std::fs::write(root.join("agent/extensions/pi-permission-system/config.json"),json!({"permissionReviewLog":false,"debugLog":false,"yoloMode":false,"promptTimeoutMs":if case.ends_with("timeout"){100}else{10000},"permission":{"*":"allow","write":if ask{"ask"}else{"allow"},"bash":"allow","external_directory":"allow","path_write":{format!("{}/*",root.join("outside").display()):if ask{"allow"}else{"deny"},hard.to_string_lossy():"deny"}}}).to_string()).unwrap();
        let extension = if case.starts_with("sandbox") {
            packages.join("pi-sandbox/index.ts")
        } else {
            packages.join("@gotgenes/pi-permission-system/src/index.ts")
        };
        assert!(extension.is_file());
        let mut cmd = Command::new(&exe);
        cmd.env_clear()
            .env("PATH", "/usr/local/bin:/usr/bin:/bin")
            .env("HOME", root.join("home"))
            .env("PI_CODING_AGENT_DIR", root.join("agent"))
            .env("PI_OFFLINE", "1")
            .env("XDG_CONFIG_HOME", root.join("config"))
            .env("XDG_CACHE_HOME", root.join("cache"))
            .current_dir(root.join("project"))
            .args([
                "--no-session",
                "--no-extensions",
                "--no-mcp",
                "--no-skills",
                "--no-prompt-templates",
                "--no-themes",
                "--no-context-files",
                "--provider",
                "fixture",
                "--model",
                "fixture",
                "-e",
            ])
            .arg(extension);
        if case.starts_with("combined") {
            cmd.arg("-e").arg(packages.join("pi-sandbox/index.ts"));
        }
        let mut client = Client::spawn(
            &mut cmd,
            Limits {
                event_capacity: 128,
                ..Limits::default()
            },
        )
        .unwrap();
        let h = client.handle();
        assert!(
            h.request(json!({"type":"get_state"}), Duration::from_secs(10))
                .await
                .unwrap()
                .success
        );
        assert!(
            h.request(
                json!({"type":"prompt","message":"Execute exactly the fixture tool"}),
                Duration::from_secs(10)
            )
            .await
            .unwrap()
            .success
        );
        let mut result: Option<Value> = None;
        let mut dialogs = 0;
        let mut last_dialog_id = None;
        tokio::time::timeout(Duration::from_secs(20), async {
            while let Some(e) = client.events.recv().await {
                if e.kind == "extension_error" {
                    panic!("{case} extension failed: {}", e.payload)
                }
                if e.kind == "extension_ui_request" {
                    if e.payload["method"] == "notify" {
                        let m = e.payload["message"].as_str().unwrap_or("");
                        assert!(!m.contains("initialization failed"), "{case}: {m}");
                    }
                    if e.payload["method"] == "select" {
                        dialogs += 1;
                        last_dialog_id = e.payload["id"].as_str().map(str::to_owned);
                        let options = e.payload["options"].as_array().unwrap();
                        println!("{case} actual options: {options:?}");
                        // Exact constants verified in installed permission-dialog.ts (not inferred labels).
                        assert!(options.contains(&json!("Yes")) && options.contains(&json!("No")));
                        if case.ends_with("timeout") {
                            continue;
                        }
                        let answer = if case.ends_with("cancel") {
                            Answer::Cancel
                        } else {
                            Answer::Value(
                                if case.ends_with("approve") {
                                    "Yes"
                                } else {
                                    "No"
                                }
                                .into(),
                            )
                        };
                        h.respond_ui(e.payload["id"].as_str().unwrap(), answer)
                            .await
                            .unwrap();
                    }
                }
                if e.kind == "tool_execution_end" {
                    result = Some(e.payload.clone());
                }
                if e.kind == "agent_settled" {
                    break;
                }
            }
        })
        .await
        .expect("security run deadline");
        if case.ends_with("timeout") {
            assert!(
                h.respond_ui(
                    last_dialog_id.as_deref().unwrap(),
                    Answer::Value("Yes".into())
                )
                .await
                .is_err()
            );
        }
        client.shutdown().await.unwrap();
        provider.kill().await.unwrap();
        let _ = provider.wait().await;
        let result = result.expect("actual tool result required");
        let allowed =
            case.ends_with("allow") || case.ends_with("approve") || case.ends_with("read");
        assert_eq!(
            result["isError"],
            !allowed,
            "{case} result {result}; root {}",
            root.display()
        );
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            if allowed && !case.ends_with("read") {
                "changed"
            } else {
                "unchanged"
            },
            "{case}: filesystem evidence"
        );
        if ask {
            assert_eq!(dialogs, 1, "{case}: genuine UI request required");
        }
        println!(
            "PASS {case}: backend result + marker verified; {}",
            root.display()
        );
    }
}
