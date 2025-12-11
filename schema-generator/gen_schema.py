import pyarrow as pa
import json
import argparse

def generate_market_schema():
    return pa.schema([
        ('symbol', pa.string()),
        ('price', pa.float64()),
        ('size', pa.uint32()),
        ('ts_ns', pa.uint64())
    ])

def main():
    parser = argparse.ArgumentParser(description="Generate Arrow Schema JSON")
    parser.add_argument('--out', type=str, default="schema.json", help="Output file")
    args = parser.parse_args()

    schema = generate_market_schema()
    
    # Extract fields metadata
    fields = []
    for field in schema:
        fields.append({
            "name": field.name,
            "type": str(field.type),
            "nullable": field.nullable
        })
    
    output = {
        "name": "MarketTick",
        "fields": fields
    }

    with open(args.out, 'w') as f:
        json.dump(output, f, indent=2)
    
    print(f"Schema generated at {args.out}")

if __name__ == "__main__":
    main()
