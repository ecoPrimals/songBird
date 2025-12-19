"""
Simple connection tester for students.

Usage:
    python -m ecoprimals_client.connect
    
Or with URL:
    SONGBIRD_URL=ws://192.168.1.144:8080 python -m ecoprimals_client.connect
"""

import asyncio
import os
import sys
from .client import SongbirdClient
from rich.console import Console

console = Console()


async def test_connection():
    """Test connection to Songbird federation."""
    
    # Get URL from environment or prompt
    url = os.getenv("SONGBIRD_URL")
    
    if not url:
        console.print("\n[yellow]No SONGBIRD_URL set.[/yellow]")
        console.print("Ask your instructor for the federation URL.\n")
        
        url = input("Enter Songbird URL (e.g., ws://192.168.1.144:8080): ").strip()
        
        if not url:
            console.print("[red]No URL provided. Exiting.[/red]")
            return False
            
    console.print()
    
    try:
        client = SongbirdClient(url)
        info = await client.connect()
        
        console.print("\n✅ [bold green]Connection successful![/bold green]\n")
        console.print("You can now submit ML tasks to the federation.")
        console.print("\nTry:")
        console.print("  [cyan]cd projects/01-mnist-digits[/cyan]")
        console.print("  [cyan]python submit.py[/cyan]")
        console.print()
        
        await client.disconnect()
        return True
        
    except Exception as e:
        console.print(f"\n❌ [red]Connection failed:[/red] {e}\n")
        console.print("[yellow]Troubleshooting:[/yellow]")
        console.print("  1. Check that Songbird is running")
        console.print("  2. Verify you're on the same network")
        console.print("  3. Check the URL is correct")
        console.print("  4. Ask your instructor for help\n")
        return False


def main():
    """Entry point for connection test."""
    success = asyncio.run(test_connection())
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()

