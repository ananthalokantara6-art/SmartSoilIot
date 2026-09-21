// queue.rs
// Antrean sederhana yang tidak bisa diubah (immutable)
// Setiap kali tambah/ambil data, kita bikin antrean BARU
// Jadi antrean lama tetap utuh

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Queue<T> {
    items: VecDeque<T>,
}

impl<T: Clone> Queue<T> {
    // Bikin antrean kosong
    pub fn new() -> Self {
        Queue {
            items: VecDeque::new(),
        }
    }

    // Tambah data ke belakang
    // PENTING: tidak mengubah antrean asli,
    // tapi mengembalikan antrean BARU
    pub fn enqueue(&self, item: T) -> Self {
        let mut baru = self.items.clone();
        baru.push_back(item);
        Queue { items: baru }
    }

    // Ambil data dari depan
    // Mengembalikan data + antrean baru (sisa)
    pub fn dequeue(&self) -> Option<(T, Self)> {
        let mut baru = self.items.clone();
        let item = baru.pop_front()?;
        Some((item, Queue { items: baru }))
    }

    // Lihat data paling depan tanpa mengambil
    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }

    // Cek antrean kosong atau tidak
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    // Hitung jumlah data di antrean
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

        #[test]
    fn test_antrean_kosong() {
        let q: Queue<i32> = Queue::new();
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
    }

    #[test]
    fn test_tambah_data() {
        let q1 = Queue::new();
        let q2 = q1.enqueue(10);

        // q1 harus tetap kosong (immutable!)
        assert_eq!(q1.len(), 0);
        // q2 harus punya 1 data
        assert_eq!(q2.len(), 1);
    }

    #[test]
    fn test_ambil_data_urutan() {
        let q = Queue::new().enqueue(10).enqueue(20).enqueue(30);

        // Ambil pertama → harus 10
        let (data1, q2) = q.dequeue().unwrap();
        assert_eq!(data1, 10);

        // Ambil kedua → harus 20
        let (data2, _) = q2.dequeue().unwrap();
        assert_eq!(data2, 20);
    }

    #[test]
    fn test_antrean_tetap_utuh() {
        let q1 = Queue::new().enqueue(1).enqueue(2);
        let q2 = q1.enqueue(3);

        // q1 harus tetap 2 data (nggak berubah)
        assert_eq!(q1.len(), 2);
        // q2 harus 3 data
        assert_eq!(q2.len(), 3);
    }
}