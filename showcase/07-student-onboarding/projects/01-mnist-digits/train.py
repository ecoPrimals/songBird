"""
MNIST Digit Classification - Student Example

This is standard PyTorch code. No special EcoPrimals APIs needed!
The federation handles all the infrastructure automatically.

Author: EcoPrimals Education Team
License: MIT
"""

import torch
import torch.nn as nn
import torch.optim as optim
from torchvision import datasets, transforms
from torch.utils.data import DataLoader
import json
import time


class SimpleNet(nn.Module):
    """
    Simple feedforward neural network for MNIST classification.
    
    Architecture:
        Input (28x28) -> Flatten -> FC(128) -> ReLU -> FC(10) -> Output
    """
    
    def __init__(self):
        super().__init__()
        self.flatten = nn.Flatten()
        self.fc1 = nn.Linear(28 * 28, 128)
        self.relu = nn.ReLU()
        self.fc2 = nn.Linear(128, 10)
        
    def forward(self, x):
        x = self.flatten(x)
        x = self.relu(self.fc1(x))
        return self.fc2(x)


def train():
    """Train MNIST classifier."""
    
    # Setup device
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"🚀 Training on: {device}")
    
    # Hyperparameters
    batch_size = 64
    epochs = 3
    learning_rate = 0.001
    
    # Data loading
    print("📦 Loading MNIST dataset...")
    transform = transforms.Compose([
        transforms.ToTensor(),
        transforms.Normalize((0.1307,), (0.3081,))
    ])
    
    train_dataset = datasets.MNIST(
        './data',
        train=True,
        download=True,
        transform=transform
    )
    
    train_loader = DataLoader(
        train_dataset,
        batch_size=batch_size,
        shuffle=True
    )
    
    test_dataset = datasets.MNIST(
        './data',
        train=False,
        download=True,
        transform=transform
    )
    
    test_loader = DataLoader(
        test_dataset,
        batch_size=batch_size,
        shuffle=False
    )
    
    print(f"   Training samples: {len(train_dataset)}")
    print(f"   Test samples: {len(test_dataset)}")
    
    # Model setup
    print("\n🧠 Building model...")
    model = SimpleNet().to(device)
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.Adam(model.parameters(), lr=learning_rate)
    
    print(f"   Parameters: {sum(p.numel() for p in model.parameters()):,}")
    
    # Training loop
    print(f"\n🏃 Training for {epochs} epochs...")
    print("=" * 60)
    
    start_time = time.time()
    training_history = []
    
    for epoch in range(epochs):
        model.train()
        total_loss = 0
        correct = 0
        total = 0
        
        for batch_idx, (data, target) in enumerate(train_loader):
            data, target = data.to(device), target.to(device)
            
            optimizer.zero_grad()
            output = model(data)
            loss = criterion(output, target)
            loss.backward()
            optimizer.step()
            
            total_loss += loss.item()
            pred = output.argmax(dim=1)
            correct += pred.eq(target).sum().item()
            total += target.size(0)
            
            # Progress updates
            if batch_idx % 100 == 0:
                current_acc = 100.0 * correct / total
                print(f"   Epoch {epoch+1}/{epochs} | "
                      f"Batch {batch_idx}/{len(train_loader)} | "
                      f"Accuracy: {current_acc:.2f}%")
        
        # Epoch summary
        train_accuracy = 100.0 * correct / total
        avg_loss = total_loss / len(train_loader)
        
        # Test evaluation
        model.eval()
        test_correct = 0
        test_total = 0
        
        with torch.no_grad():
            for data, target in test_loader:
                data, target = data.to(device), target.to(device)
                output = model(data)
                pred = output.argmax(dim=1)
                test_correct += pred.eq(target).sum().item()
                test_total += target.size(0)
        
        test_accuracy = 100.0 * test_correct / test_total
        
        epoch_result = {
            "epoch": epoch + 1,
            "train_loss": avg_loss,
            "train_accuracy": train_accuracy,
            "test_accuracy": test_accuracy
        }
        training_history.append(epoch_result)
        
        print(f"\n   📊 Epoch {epoch+1} Results:")
        print(f"      Train Loss: {avg_loss:.4f}")
        print(f"      Train Accuracy: {train_accuracy:.2f}%")
        print(f"      Test Accuracy: {test_accuracy:.2f}%")
        print("=" * 60)
    
    training_time = time.time() - start_time
    
    # Final results
    final_results = {
        "model": "SimpleNet",
        "dataset": "MNIST",
        "final_train_accuracy": train_accuracy,
        "final_test_accuracy": test_accuracy,
        "final_loss": avg_loss,
        "epochs": epochs,
        "batch_size": batch_size,
        "learning_rate": learning_rate,
        "training_time_seconds": training_time,
        "device": str(device),
        "history": training_history
    }
    
    print(f"\n✅ Training complete!")
    print(f"   Final test accuracy: {test_accuracy:.2f}%")
    print(f"   Total time: {training_time:.1f}s")
    
    # Save results
    with open("results.json", "w") as f:
        json.dump(final_results, f, indent=2)
    
    print(f"\n💾 Results saved to: results.json")
    
    return final_results


if __name__ == "__main__":
    print("\n" + "=" * 60)
    print("  MNIST DIGIT CLASSIFICATION")
    print("  EcoPrimals Student Project")
    print("=" * 60 + "\n")
    
    results = train()
    
    print("\n" + "=" * 60)
    print("  SESSION COMPLETE")
    print("=" * 60 + "\n")

