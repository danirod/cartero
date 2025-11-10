---
title: "Bumping the version to 25"
description: "Cartero is switching away from the 0.x version conventions."
pubDate: "Nov 10 2025"
heroImage: "../../assets/post-images/bump-to-25.jpg"
---

Cartero is adopting a new versioning scheme based on the ISO 8601 year as
part of the version number. This means that the next version of Cartero will
be tagged as 25.0, instead of the previous target, 0.3.

Many popular software in the free and open source world uses this versioning
scheme, such as Ubuntu, LibreOffice or the KDE Apps Suite, so it is expected
that this change will not be confusing for most users.

The version number of Cartero will usually consist of two numbers separated by
a dot. The first number will represent the year of release according to the
Gregorian calendar, the one that ISO 8601 uses. The second number is the
counter of releases made so far the current year. So 0 is the first one, 1
is the second, 2 the third...

Actually, there have been already 7 releases of Cartero since 2025-01-01, and
Cartero 25.0 would actually have been the 8th one. However, I considered that
bumping the version to 25.8 would have been more confusing because some
people might wonder where would the previous 7 releases be.

Also, it's kind of audacious to switch the versioning scheme in November, just
a few weeks away from 2026, where starting a fresh year with Cartero 26.0 would
have made more sense, but this is also part of the reason why I am making this
change now.

And the full story is: I just want to release.

Look, working with milestones provides a safe framework that enforces iterative
work and some sort of quality on the software that is being made. However,
milestones and iterations are made for corporate software. But Cartero is not
corporate software, it's just a free and open source tool made in community.

Previously, every version of Cartero (0.1.0, 0.2.0, 0.3.0...) was aligned with
a milestone full of tasks and goals to complete before moving on to the next
version. Each milestone would get us closer to Cartero 1.0.0, which would mean
that the software is feature complete.

But this has a few drawbacks:

* **Iterative work requires consistency.** And consistency is something that
  right now I cannot guarantee. The last couple of months have been difficult
  for me, both in the personal and professional levels. Thus, the development
  speed has slowed down significantly, which is something that iterative and
  milestone-based frameworks take bad, because they are made for the corporate
  world, where either you work full-time or get the boot.

* **A slow work environment starves features.** Cartero has had a lot of
  features in the nightly releases for a couple of months already. It's just
  that all this work has been waiting for a release. These are very cool
  features, such as proxies, new formats, and QoL enhancements, that nobody is
  able to see, because the task list says that there is work to do before the
  world can see the next release. This is basically the opposite to what
  free and open source models such as [Release early, release often][rero]
  have traditionally proposed.

* **No fun allowed**. Also, when milestones are too strict, they stop fun and
  goofy features. For instance, sometimes I am not in the mood to work on
  the things that the task list says I am supposed to do, after a long day of
  work, because my head hurts. But sometimes I get creative and want to explore
  things that are not part of the milestone but that look fun anyway, such as
  re-designing a window or adding support for user themes.

* **Who chooses what constitutes complete?** For some people, Cartero may
  already do what they expect. Maybe they just want a graphical cURL and it
  fills their needs. Even if for me it clearly doesn't.

Switching to a calendar based release system reduces these pain points.
Additionally, I will use this opportunity to simplify the release engineering
of the repository. I will try to work from the trunk branch more often and
will stop making release branches such as `stable/0.2` or `stable/0.3`, since
they will not be required anymore.

I admit that this is not ideal. For instance, not following a task list and
focusing on releasing often might cause the quality of the software to go
down, or important features to be snoozed often. I will do my best to keep
the bar up. Also, in regards to the quality, you know that the best way to
avoid buggy software is to report bugs when they happen.

Cartero 25.0 is very close to release. Keep an eye for updates, it will not
take more than a day or two.

[rero]: https://en.wikipedia.org/wiki/Release_early,_release_often
