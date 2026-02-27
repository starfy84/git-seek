pub struct PresetParam {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
    pub default: Option<&'static str>,
    /// If true, the parameter value is substituted directly into the query string
    /// (for edge arguments like `limit`). If false, it is passed as a Trustfall
    /// query variable (for `@filter` values).
    pub inline: bool,
}

pub struct Preset {
    pub name: &'static str,
    pub description: &'static str,
    pub query: &'static str,
    pub params: &'static [PresetParam],
}

static PRESETS: &[Preset] = &[
    Preset {
        name: "recent-commits",
        description: "Show recent commits",
        query: r#"{
  repository {
    commits(limit: $limit) {
      hash @output
      message @output
      author @output
      date @output
    }
  }
}"#,
        params: &[PresetParam {
            name: "limit",
            description: "Maximum number of commits to show",
            required: false,
            default: Some("10"),
            inline: true,
        }],
    },
    Preset {
        name: "branches",
        description: "List all branches with their latest commit",
        query: r#"{
  repository {
    branches {
      name @output
      commit {
        hash @output
        message @output
      }
    }
  }
}"#,
        params: &[],
    },
    Preset {
        name: "tags",
        description: "List all tags with their commit",
        query: r#"{
  repository {
    tags {
      name @output
      message @output
      commit {
        hash @output
      }
    }
  }
}"#,
        params: &[],
    },
    Preset {
        name: "commits-by-author",
        description: "Show commits by a specific author",
        query: r#"{
  repository {
    commits {
      author @output @filter(op: "=", value: ["$author"])
      hash @output
      message @output
      date @output
    }
  }
}"#,
        params: &[PresetParam {
            name: "author",
            description: "Author name to filter by",
            required: true,
            default: None,
            inline: false,
        }],
    },
    Preset {
        name: "search-commits",
        description: "Search commit messages by regex pattern",
        query: r#"{
  repository {
    commits {
      message @output @filter(op: "regex", value: ["$pattern"])
      hash @output
      author @output
      date @output
    }
  }
}"#,
        params: &[PresetParam {
            name: "pattern",
            description: "Regex pattern to search for in commit messages",
            required: true,
            default: None,
            inline: false,
        }],
    },
];

pub fn all_presets() -> &'static [Preset] {
    PRESETS
}

pub fn find_preset(name: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_all_presets_returns_five() {
        assert_eq!(all_presets().len(), 5);
    }

    #[test]
    fn test_find_preset_exists() {
        let preset = find_preset("recent-commits");
        assert!(preset.is_some());
        assert_eq!(preset.unwrap().name, "recent-commits");
    }

    #[test]
    fn test_find_preset_not_found() {
        assert!(find_preset("nonexistent").is_none());
    }

    #[test]
    fn test_all_preset_names_unique() {
        let names: HashSet<&str> = all_presets().iter().map(|p| p.name).collect();
        assert_eq!(names.len(), all_presets().len());
    }

    #[test]
    fn test_all_preset_queries_nonempty() {
        for preset in all_presets() {
            assert!(
                !preset.query.is_empty(),
                "Preset '{}' has an empty query",
                preset.name
            );
        }
    }

    #[test]
    fn test_recent_commits_has_limit_param_with_default() {
        let preset = find_preset("recent-commits").unwrap();
        assert_eq!(preset.params.len(), 1);
        let param = &preset.params[0];
        assert_eq!(param.name, "limit");
        assert!(!param.required);
        assert_eq!(param.default, Some("10"));
    }

    #[test]
    fn test_commits_by_author_has_required_param() {
        let preset = find_preset("commits-by-author").unwrap();
        assert_eq!(preset.params.len(), 1);
        let param = &preset.params[0];
        assert_eq!(param.name, "author");
        assert!(param.required);
        assert_eq!(param.default, None);
    }

    #[test]
    fn test_branches_has_no_params() {
        let preset = find_preset("branches").unwrap();
        assert!(preset.params.is_empty());
    }
}
