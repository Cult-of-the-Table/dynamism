use crate::model::Point;

const CONTENTS: &[&str] = &[
    "The quick brown fox jumps over the lazy dog. This sample text demonstrates basic embedding visualization for document retrieval systems.",
    "Machine learning models process input data through multiple layers of transformation, each extracting increasingly abstract features.",
    "Vector embeddings capture semantic meaning in high-dimensional space, allowing similarity comparisons between documents.",
    "The architecture consists of an encoder-decoder structure with attention mechanisms that weigh the importance of different input tokens.",
    "Natural language processing has evolved from rule-based systems to neural approaches that learn representations from data.",
    "Dimensionality reduction techniques like t-SNE and UMAP help visualize high-dimensional embeddings in two or three dimensions.",
    "Transfer learning enables models pretrained on large corpora to be fine-tuned for domain-specific downstream tasks.",
    "The cosine similarity metric measures the angle between two vectors, providing a scale-invariant measure of semantic relatedness.",
    "Attention mechanisms allow models to focus on relevant parts of the input sequence when producing each element of the output.",
    "Tokenization splits text into subword units, balancing vocabulary size against the ability to represent any input string.",
    "Gradient descent optimization adjusts model parameters iteratively to minimize a loss function over the training data.",
    "Regularization techniques such as dropout and weight decay prevent neural networks from overfitting to the training set.",
    "Batch normalization stabilizes training by normalizing intermediate activations within each mini-batch of data.",
    "The transformer architecture replaced recurrent connections with self-attention, enabling parallel computation across sequence positions.",
    "Cross-attention allows a decoder to attend over encoder outputs, bridging the gap between source and target representations.",
    "Positional encodings inject sequence order information into transformer models that otherwise process tokens in parallel.",
    "Beam search decoding explores multiple candidate output sequences simultaneously, improving quality over greedy selection.",
    "Fine-tuning adapts a pretrained model to a specific task by continuing training on a smaller, task-specific dataset.",
    "The embedding space learned by language models exhibits linear relationships that enable analogy reasoning through vector arithmetic.",
    "Semantic search uses embedding similarity rather than keyword matching to retrieve documents relevant to a user query.",
    "Contrastive learning trains models to distinguish similar from dissimilar pairs, producing well-structured embedding spaces.",
    "Knowledge distillation transfers learned representations from a large teacher model to a smaller, more efficient student model.",
    "Multi-head attention computes several attention functions in parallel, allowing the model to attend to information from different subspaces.",
    "The softmax function converts raw logits into a probability distribution, commonly used as the final layer for classification tasks.",
    "Curriculum learning presents training examples in a structured order from easy to difficult, accelerating convergence.",
    "Data augmentation creates synthetic training examples through transformations, improving model robustness without collecting more data.",
    "The perplexity metric evaluates language model quality by measuring how well the model predicts held-out text.",
    "Reinforcement learning from human feedback aligns model outputs with human preferences by optimizing a learned reward model.",
    "Quantization reduces model precision from floating point to lower-bit representations, enabling faster inference with minimal quality loss.",
    "Retrieval-augmented generation combines a knowledge store with a language model, grounding outputs in retrieved evidence.",
];

const X_VALUES: &[f64] = &[
    -3.2, 1.4, 2.8, -0.5, 4.1, -1.9, 3.6, 0.7, -2.3, 5.0,
    -4.5, 1.1, -0.8, 3.3, 2.0, -1.2, 4.7, -3.8, 0.3, -2.7,
    1.8, -0.1, 3.9, -1.6, 2.5, -3.0, 0.9, 4.4, -2.1, 1.6,
];

const Y_VALUES: &[f64] = &[
    2.1, -1.5, 3.7, 0.4, -2.8, 1.9, -0.6, 4.2, -3.1, 2.4,
    0.8, -4.0, 3.2, -1.7, 1.3, 2.9, -0.3, -2.5, 4.6, -3.4,
    0.1, 1.7, -4.3, 2.6, -0.9, 3.8, -1.1, 4.0, -2.0, 3.5,
];

pub fn generate_demo_data() -> Vec<Point> {
    (0..30)
        .map(|i| Point {
            title: format!("doc-{:03}", i + 1),
            x: X_VALUES[i],
            y: Y_VALUES[i],
            content: CONTENTS[i].to_string(),
        })
        .collect()
}
