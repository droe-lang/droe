import SwiftUI
import AVFoundation
import CoreLocation

struct ContentView: View {
    @State private var showingForm = false
    @StateObject private var locationManager = LocationManager()
    
    var body: some View {
        NavigationView {
            MainscreenView()
            .navigationTitle("MyApp")
        }
        .onAppear {
            self.requestPermissions()
        }
    }
    
    func requestPermissions() {
        AVCaptureDevice.requestAccess(for: .video) { _ in }
    }
    
    func openCamera() {
        // Camera implementation
        // In a real app, this would present a camera view
    }
    
    func getCurrentLocation() {
        locationManager.requestLocation()
    }
    
}

class LocationManager: NSObject, ObservableObject, CLLocationManagerDelegate {
    private let manager = CLLocationManager()
    @Published var location: CLLocation?
    
    override init() {
        super.init()
        manager.delegate = self
        manager.desiredAccuracy = kCLLocationAccuracyBest
    }
    
    func requestLocation() {
        manager.requestWhenInUseAuthorization()
        manager.requestLocation()
    }
    
    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        location = locations.first
    }
    
    func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
        print("Location error: \(error)")
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}