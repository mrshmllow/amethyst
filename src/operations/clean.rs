use std::process::Command;

use crate::crash;
use crate::info;
use crate::internal::commands::ShellCommand;
use crate::internal::config;
use crate::internal::error::SilentUnwrap;
use crate::internal::exit_code::AppExitCode;
use crate::log;
use crate::prompt;
use crate::Options;

/// Help the user in clearing orphaned packages and pacman cache.
pub fn clean(options: Options) {
    let verbosity = options.verbosity;
    let noconfirm = options.noconfirm;

    // Check for orphaned packages
    let orphaned_packages = ShellCommand::pacman()
        .arg("-Qdtq")
        .wait_with_output()
        .silent_unwrap(AppExitCode::PacmanError);

    if orphaned_packages.stdout.as_str().is_empty() {
        // If no orphaned packages found, do nothing
        info!("No orphaned packages found");
    } else {
        // Prompt users whether to remove orphaned packages
        info!(
            "Removing orphans would uninstall the following packages: \n{}",
            &orphaned_packages.stdout
        );
        let cont = prompt!(default false, "Continue?");
        if !cont {
            // If user doesn't want to continue, break
            info!("Exiting");
            std::process::exit(AppExitCode::PacmanError as i32);
        }

        // Build pacman args
        let mut pacman_args = vec!["-Rns"];
        if noconfirm {
            pacman_args.push("--noconfirm");
        }

        // Collect orphaned packages into a vector
        let orphaned_packages_vec = orphaned_packages.stdout.split('\n').collect::<Vec<&str>>();
        for package in &orphaned_packages_vec {
            if !package.is_empty() {
                pacman_args.push(package);
            }
        }

        if verbosity >= 1 {
            log!("Removing orphans: {:?}", orphaned_packages_vec);
        }

        // Remove orphaned packages
        let pacman_result = ShellCommand::pacman()
            .elevated()
            .args(pacman_args)
            .wait()
            .silent_unwrap(AppExitCode::PacmanError);

        if pacman_result.success() {
            // If pacman succeeded, notify user
            info!("Successfully removed orphans");
        } else {
            // If pacman failed, crash
            crash!(AppExitCode::PacmanError, "Failed to remove orphans",);
        }
    }

    // Prompt the user whether to clear the Amethyst cache
    let clear_ame_cache = prompt!(default false, "Clear Amethyst's internal PKGBUILD cache?");
    if clear_ame_cache {
        // Remove ~/.cache/ame
        Command::new("rm")
            .arg("-rf")
            .arg("~/.cache/ame")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    // Prompt the user whether to clear cache or not
    let clear_pacman_cache = if noconfirm {
        true
    } else {
        prompt!(default false, "Also clear pacman's package cache?")
    };

    if clear_pacman_cache {
        // Build pacman args
        let mut pacman_args = vec!["-Sc"];
        if noconfirm {
            pacman_args.push("--noconfirm");
        }

        // Build paccache args
        let mut paccache_args = vec!["-r"];
        if noconfirm {
            paccache_args.push("--noconfirm");
        }

        if verbosity >= 1 {
            log!("Clearing using `paccache -r`");
        }

        // Clear pacman's cache (keeping latest 3 versions of installed packages)
        Command::new(config::read().bin.sudo.unwrap_or_default())
            .arg("paccache")
            .args(paccache_args)
            .spawn()
            .unwrap_or_else(|e| {
                crash!(
                    AppExitCode::PacmanError,
                    "Couldn't clear cache using `paccache -r`, {}",
                    e,
                )
            })
            .wait()
            .unwrap();

        if verbosity >= 1 {
            log!("Clearing using `pacman -Sc`");
        }

        // Clear pacman's cache (keeping only installed packages)
        let pacman_result = ShellCommand::pacman()
            .elevated()
            .args(pacman_args)
            .wait()
            .silent_unwrap(AppExitCode::PacmanError);

        if pacman_result.success() {
            // If pacman succeeded, notify user
            info!("Successfully cleared package cache");
        } else {
            // If pacman failed, crash
            crash!(AppExitCode::PacmanError, "Failed to clear package cache",);
        }
    }
}
