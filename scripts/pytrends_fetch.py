#!/usr/bin/env python3
"""Fetch Google Trends data via PyTrends (no API key required). Outputs JSON to stdout."""

import json
import re
import sys
import time


def parse_interest(value) -> int:
    if value is None:
        return 50
    if isinstance(value, (int, float)):
        return max(0, min(100, int(value)))
    text = str(value).strip().replace("%", "").replace("+", "")
    match = re.search(r"\d+", text)
    if not match:
        return 50
    return max(0, min(100, int(match.group())))


def main() -> None:
    seed = sys.argv[1] if len(sys.argv) > 1 else "OTC penny stocks"
    keyword = seed.strip() or "OTC penny stocks"

    try:
        from pytrends.request import TrendReq
    except ImportError:
        print("[]", flush=True)
        sys.exit(1)

    topics = []
    try:
        pytrends = TrendReq(hl="en-US", tz=360, retries=2, backoff_factor=0.4)
        pytrends.build_payload([keyword], timeframe="now 7-d", geo="US")

        interest = pytrends.interest_over_time()
        if interest is not None and not interest.empty and keyword in interest.columns:
            avg = int(interest[keyword].mean())
            topics.append(
                {"topic": keyword, "interest": max(0, min(100, avg)), "source": "pytrends"}
            )

        related = pytrends.related_queries()
        bucket = related.get(keyword) if isinstance(related, dict) else None
        if bucket is not None:
            rising = bucket.get("rising")
            if rising is not None and not rising.empty:
                for _, row in rising.head(8).iterrows():
                    topics.append(
                        {
                            "topic": str(row["query"]),
                            "interest": parse_interest(row.get("value")),
                            "source": "pytrends",
                        }
                    )
            top = bucket.get("top")
            if top is not None and not top.empty and len(topics) < 4:
                for _, row in top.head(4).iterrows():
                    topics.append(
                        {
                            "topic": str(row["query"]),
                            "interest": parse_interest(row.get("value")),
                            "source": "pytrends",
                        }
                    )
    except Exception:
        topics = []

    if not topics:
        time.sleep(1.5)
        try:
            pytrends = TrendReq(hl="en-US", tz=360, retries=2, backoff_factor=0.4)
            pytrends.build_payload(["penny stocks"], timeframe="now 7-d", geo="US")
            related = pytrends.related_queries()
            rising = related.get("penny stocks", {}).get("rising")
            if rising is not None and not rising.empty:
                for _, row in rising.head(6).iterrows():
                    topics.append(
                        {
                            "topic": str(row["query"]),
                            "interest": parse_interest(row.get("value")),
                            "source": "pytrends",
                        }
                    )
        except Exception:
            topics = []

    if not topics:
        try:
            time.sleep(1.5)
            pytrends = TrendReq(hl="en-US", tz=360, retries=2, backoff_factor=0.4)
            trending = pytrends.trending_searches(pn="united_states")
            if trending is not None and not trending.empty:
                for item in trending.head(8).iloc[:, 0].tolist():
                    text = str(item).strip()
                    if text:
                        topics.append(
                            {"topic": text, "interest": 55, "source": "pytrends_trending"}
                        )
        except Exception:
            topics = []

    # Deduplicate by topic, keep highest interest
    deduped = {}
    for item in topics:
        key = item["topic"].lower()
        if key not in deduped or item["interest"] > deduped[key]["interest"]:
            deduped[key] = item

    result = list(deduped.values())[:8]
    print(json.dumps(result), flush=True)
    if not result:
        sys.exit(1)


if __name__ == "__main__":
    main()
