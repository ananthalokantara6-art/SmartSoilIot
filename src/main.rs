// main.rs
// Program untuk mencoba antrean kita

mod queue;  // pakai file queue.rs

use queue::Queue;

fn main() {
    println!("=== Coba Antrean Immutable ===\n");

    // Bikin antrean kosong
    let antrean1 = Queue::new();
    println!("Antrean awal: {} data", antrean1.len());

    // Tambah data satu-satu
    // Perhatikan: antrean LAMA tetap ada
    let antrean2 = antrean1.enqueue(10);
    let antrean3 = antrean2.enqueue(20);
    let antrean4 = antrean3.enqueue(30);

    println!("\nSetelah tambah 3 data:");
    println!("  antrean1 = {} data (masih kosong)", antrean1.len());
    println!("  antrean2 = {} data", antrean2.len());
    println!("  antrean3 = {} data", antrean3.len());
    println!("  antrean4 = {} data", antrean4.len());

        // Ambil data dari depan (yang pertama masuk, keluar duluan)
    println!("\nAmbil data dari depan:");
    if let Some((data, sisa)) = antrean4.dequeue() {
        println!("  Data yang diambil: {}", data);
        println!("  Sisa di antrean: {} data", sisa.len());

        // Lihat data depan dari SISA (bukan antrean4)
        if let Some(depan) = sisa.peek() {
            println!("\nData paling depan (tanpa diambil): {}", depan);
        }
    }
}