# Contributing

When contributing to this repository, please first discuss the change you wish to make via issue and via the forum https://forum.duniter.org before making a change.

Please note we have a specific workflow, please follow it in all your interactions with the project.

## Workflow

1. You must create a different issue for each feature you wish to develop, assign this issue yourself, then on the issue page click on the button "create a merge request".
2. Gitlab will then create a branch dedicated to the issue as well as a MR in the WIP state that will merge this branch into the default branch (dev).
3. Please specify the crate concerned in the issue labels.
4. Never contribute to a branch whose issue has not been assigned to you! the contributor can make a git rebase at any time and your commit would be lost !
5. Before you pusher your commit: 
  a. Apply fmt (gitlab-ci will make sure it is ok!)
  b. Verify that ALL tests always pass.
  c. Made a git rebase to make your history clean.

## Merge Process

1. Ensure any install or build dependencies are removed before the end of the layer when doing a 
   build.
2. Update the README.md with details of changes to the interface, this includes new environment 
   variables, exposed ports, useful file locations and container parameters.
3. Increase the version numbers in any examples files and the README.md to the new version that this
   Pull Request would represent. The versioning scheme we use is [SemVer](http://semver.org/).
4. You may merge the Merge Request in once you have the sign-off of two other developers, or if you 
   do not have permission to do that, you may request the second reviewer to merge it for you.