//! Revised G1: permission interception only. No OS containment claim or sandbox package.
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
#[ignore = "explicit installed Pi/security packages + loopback + permission package prerequisites"]
async fn installed_security_negative_suite() {
    let exe = std::env::var("PI_VISUAL_TEST_PI").expect("explicit Pi path");
    let packages = PathBuf::from(
        std::env::var("PI_VISUAL_TEST_PACKAGES").expect("explicit installed package directory"),
    );
    for case in [
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
        "permission-session-approve",
        "permission-external-write-allow",
        "permission-external-write-deny",
        "permission-external-ask-deny",
        "permission-external-ask-approve",
        "permission-external-read-deny",
        "permission-external-read-allow",
        "permission-python-deny",
        "permission-python-approve",
        "permission-shell-script-deny",
        "permission-shell-script-approve",
        "permission-env-deny",
        "permission-sensitive",
        "permission-sensitive-read-deny",
        "permission-bash-remove-deny",
        "permission-mcp-allow",
        "permission-mcp-opaque-allow",
        "permission-mcp-opaque-deny",
        "permission-mcp-deny",
        "permission-mcp-path-deny",
        "permission-mcp-ask-deny",
        "permission-mcp-ask-approve",
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
        let mut target =
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
        let indirect = case.contains("python") || case.contains("shell-script");
        if indirect || case.contains("env") {
            target = outside.clone();
        }
        if case.contains("sensitive") {
            target = root.join("project/.ssh/test-key");
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(&target, "unchanged").unwrap();
        }
        if case == "permission-mcp-deny" || (case.contains("mcp") && case.contains("ask")) {
            target = inside.clone();
        }
        if case.contains("remove") {
            target = hard.clone();
        }
        if case.contains("opaque") {
            target = outside.clone();
        }
        if case.contains("external") {
            target = outside.clone();
        }
        let mcp = case.contains("mcp");
        let bash = case.contains("bash");
        let mut plan = if bash {
            json!({"name":"bash","arguments":{"command":format!("printf changed > '{}'",target.display())}})
        } else if case.contains("read") {
            json!({"name":"read","arguments":{"path":target}})
        } else {
            json!({"name":"write","arguments":{"path":target,"content":"changed"}})
        };
        if case.contains("remove") {
            plan =
                json!({"name":"bash","arguments":{"command":format!("rm -- {}",hard.display())}});
        }
        if indirect {
            let script = root.join(if case.contains("python") {
                "project/opaque.py"
            } else {
                "project/opaque.sh"
            });
            let source = if case.contains("python") {
                "from pathlib import Path\np=Path.cwd().parent/'outside'/'marker.txt'\np.write_text('changed')\n"
            } else {
                "printf changed > ../outside/marker.txt\n"
            };
            std::fs::write(&script, source).unwrap();
            plan = json!({"name":"bash","arguments":{"command":format!("{} {}",if case.contains("python"){ "python3" }else{"bash"},script.display())}});
        }
        if case.contains("env") {
            plan = json!({"name":"bash","arguments":{"command":format!("env sh -c 'printf changed > {}'",outside.display())}});
        }
        if mcp {
            plan = json!({"name":"mcp__fixture__write_marker","arguments":{"path":target,"content":"changed"}});
        }
        if case.contains("mcp-opaque") {
            plan = json!({"name":"mcp__fixture__opaque_write","arguments":{}});
        }
        if case.contains("session") {
            plan = json!([
                plan.clone(),
                plan,
                json!({"name":"write","arguments":{"path":hard,"content":"changed"}})
            ]);
        }
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
        std::fs::create_dir_all(root.join("agent/extensions/pi-permission-system")).unwrap();
        let ask = case.contains("ask")
            || case.contains("session")
            || case.contains("python")
            || case.contains("shell-script")
            || case.contains("env");
        let mut permission = json!({"*":"allow","write":if ask&&!mcp&&!indirect&&!case.contains("env")&&!case.contains("external"){"ask"}else{"allow"},"bash":if indirect||case.contains("env"){"ask"}else{"allow"},"external_directory":"allow","mcp":if case=="permission-mcp-deny"||case=="permission-mcp-opaque-deny"{"deny"}else if mcp&&ask{"ask"}else{"allow"},"path_write":{format!("{}/*",root.join("outside").display()):if (case.contains("ask")&&!mcp)||indirect||case.contains("env"){"allow"}else{"deny"},hard.to_string_lossy():"deny",format!("{}/*",root.join("project/.ssh").display()):"deny"}});
        // Denied path stays denied even when approved opaque code later touches it.
        if case.contains("external") {
            permission["path_write"][format!("{}/*", root.join("outside").display())] =
                "allow".into();
            permission[if case.contains("read") {
                "external_directory_read"
            } else {
                "external_directory_write"
            }] = json!(if case.contains("ask") {
                "ask"
            } else if case.ends_with("allow") {
                "allow"
            } else {
                "deny"
            });
        }
        permission["path_read"] =
            json!({format!("{}/*",root.join("project/.ssh").display()):"deny"});
        if indirect {
            permission["path_write"][format!("{}/*", root.join("outside").display())] =
                "deny".into();
        }
        std::fs::write(root.join("agent/extensions/pi-permission-system/config.json"),json!({"permissionReviewLog":false,"debugLog":false,"yoloMode":false,"promptTimeoutMs":if case.ends_with("timeout"){100}else{10000},"permission":permission}).to_string()).unwrap();
        if mcp {
            std::fs::write(root.join("agent/mcp.json"),json!({"mcpServers":{"fixture":{"command":"python3","args":[concat!(env!("CARGO_MANIFEST_DIR"),"/../../tests/fixtures/permission-mcp.py"),root],"exposure":"direct"}}}).to_string()).unwrap();
        }
        let extension = packages.join("@gotgenes/pi-permission-system/src/index.ts");
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
        if mcp {
            cmd.arg("-e").arg("builtin:mcp");
        } else {
            cmd.arg("--no-mcp");
        }
        let mut client = Client::spawn(
            &mut cmd,
            Limits {
                event_capacity: 128,
                ..Limits::default()
            },
        )
        .unwrap();
        let mut h = client.handle();
        assert!(
            h.request(json!({"type":"get_state"}), Duration::from_secs(10))
                .await
                .unwrap()
                .success
        );
        let mut result: Option<Value> = None;
        let mut outcomes = Vec::new();
        let mut dialogs = 0;
        let mut last_dialog_id = None;
        for round in 0..if case.contains("session") { 2 } else { 1 } {
            if round > 0 {
                client.restart(&mut cmd).await.unwrap();
                h = client.handle();
            }
            assert!(
                h.request(
                    json!({"type":"prompt","message":"Execute exactly the fixture tool"}),
                    Duration::from_secs(10)
                )
                .await
                .unwrap()
                .success
            );
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
                            assert!(
                                options.contains(&json!("Yes")) && options.contains(&json!("No"))
                            );
                            if case.ends_with("timeout") {
                                continue;
                            }
                            let answer = if case.contains("session") {
                                let expected = format!(
                                    "Yes, allow write \"{}/*\" for this session",
                                    root.join("project").display()
                                );
                                assert!(
                                    options.contains(&json!(expected)),
                                    "session label must match installed semantics: {options:?}"
                                );
                                Answer::Value(expected)
                            } else if case.ends_with("cancel") {
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
                        outcomes.push(e.payload.clone());
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
        }
        client.shutdown().await.unwrap();
        provider.kill().await.unwrap();
        let _ = provider.wait().await;
        let result = result.expect("actual tool result required");
        let allowed =
            case.ends_with("allow") || case.ends_with("approve") || case.ends_with("read");
        if case.contains("session") {
            assert_eq!(outcomes.len(), 6);
            assert_eq!(outcomes[0]["isError"], false);
            assert_eq!(outcomes[1]["isError"], false);
            assert_eq!(outcomes[2]["isError"], true);
            assert_eq!(outcomes[3]["isError"], false);
            assert_eq!(outcomes[4]["isError"], false);
            assert_eq!(outcomes[5]["isError"], true);
            assert_eq!(std::fs::read_to_string(&hard).unwrap(), "unchanged");
        }

        assert_eq!(
            result["isError"],
            if case.contains("session") {
                true
            } else {
                !allowed
            },
            "{case} result {result}; root {}",
            root.display()
        );
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            if (allowed || case.contains("session")) && !case.contains("read") {
                "changed"
            } else {
                "unchanged"
            },
            "{case}: filesystem evidence"
        );
        if mcp {
            let receipt = root.join("mcp-executions.jsonl");
            assert_eq!(
                receipt.exists(),
                allowed,
                "denied MCP must never execute server tool"
            );
        }
        if ask {
            assert_eq!(
                dialogs,
                if case.contains("session") { 2 } else { 1 },
                "{case}: genuine UI request required, session grant must not survive process restart"
            );
        }
        if (indirect || case.contains("opaque")) && allowed {
            println!(
                "LIMIT {case}: approved opaque execution modified a marker denied to direct path-bearing tools; no OS containment"
            );
        }
        println!(
            "PASS {case}: backend result + marker verified; {}",
            root.display()
        );
    }
}
