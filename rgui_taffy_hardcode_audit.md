# RGUI Taffy Hardcode Audit

Taffy is integrated as the production layout authority for document flow.
Runtime paint, hit testing, snapshots, scrollbars, and overlays consume Taffy
layout boxes rather than estimating geometry independently.

WidgetMetrics is the source of widget sizing policy. New widget dimensions and
scrollbar geometry should be added to theme metrics or resolved style data, not
as raw literals in runtime layout or paint paths.

Paint pass still contains hardcoded visual geometry in legacy painter code, so
the hardcode policy tests keep this audit file as traceability input while the
remaining constants move behind theme tokens and widget metrics.
