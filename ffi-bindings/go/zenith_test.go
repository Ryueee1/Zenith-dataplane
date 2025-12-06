package zenith

import (
	"testing"
)

func TestClientCreation(t *testing.T) {
	client, err := NewClient(1024)
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}
	defer client.Close()

	if client.closed {
		t.Error("Client should not be closed after creation")
	}
}

func TestClientClose(t *testing.T) {
	client, err := NewClient(1024)
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}

	err = client.Close()
	if err != nil {
		t.Errorf("Close failed: %v", err)
	}

	if !client.closed {
		t.Error("Client should be marked as closed")
	}

	// Double close should be safe
	err = client.Close()
	if err != nil {
		t.Errorf("Second close failed: %v", err)
	}
}

func TestGetStats(t *testing.T) {
	client, err := NewClient(1024)
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}
	defer client.Close()

	stats, err := client.GetStats()
	if err != nil {
		t.Errorf("GetStats failed: %v", err)
	}

	if stats == nil {
		t.Error("Stats should not be nil")
	}
}
