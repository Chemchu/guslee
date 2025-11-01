---
title: "Waternology, a messy mess"
date: "2024-09-15"
description: "My experience with Sopra Steria and FACSA"
tags:
  - work
  - engineering
---

# The project was a mess

I honestly think most of the problems were caused because it was really hard to
develop and iterate over the data we had. The project was a big migration from
an old system to a newer one. From BASIC6 to Angular. Huge change. One of our
most concerning problems were the way we handled data. Both storing it and
retrieving it was a hassle. Every new feature had a huge push back because of
the complexity of the date we handled. You must be thinking that we handled
crazy amounts of rows with thousands of columns and very complex joins, but no.
The problem were mostly with the database. Well, no. With the database changes.
You see, at the start we had a blank database with 0 tables. We had to create
them over the sprints. That made the whole system very unreliable. Each new
feature required a new database analysis. New FK and new constraints were added
to already existing columns. That was a pain because we lost most of our time
fixing this issues instead of implementing the feature. Redifining APIs,
migrating tables, changing tests, notifying everyone about our change. All of
that because nobody cared about the correct and robust definition of the data.

I know it's impossible to know the perfect database model for any application,
especially if you work in sprints and iterate over the past features and
changes. BUT, let me say it again. We were migrating an old system. We already
knew how the data looked. We knew EVERYTHING. We just had to create a new
database model based on the old one. A job for a database engineer or something.
But no, nobody cared about that, not even my fellow coworkers.

But Gustavo, in doing so you guys would have needed some sprints only for the
definition of the database. You sure didn't think of that, right? Well, yes. Of
course I did. My boss (C*rmen) had a whole YEAR to analyze the system and
understand their product. She needed to get the database definition DONE by
then. Maybe by hiring somebody that knew what to do. But they had that time to
do it. A whole year for that. But they didn't do it because nobody knows shit.

I honestly think that I can implement Waternology from scratch all by myself.
That's how confident I am looking back at it. I might have needed 5 years, but
the product would have been finished
